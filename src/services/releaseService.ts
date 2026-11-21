import { useUpdateStore } from "../stores/updateStore";

export interface GitHubReleaseItem {
  tag: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  body: string;
  isPrerelease?: boolean;
}

const RELEASES_API_URL = "https://api.github.com/repos/Lelequyngocquy/desktop-bubble-chat/releases?per_page=50";
const LATEST_JSON_URL = "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest/download/latest.json";
const RELEASES_HTML_URL = "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases";
const RELEASE_CACHE_TTL_MS = 10 * 60 * 1000;

let cachedReleases: { items: GitHubReleaseItem[]; fetchedAt: number } | null = null;

function isAbortError(err: unknown): boolean {
  return err instanceof DOMException ? err.name === "AbortError" : Boolean(err && typeof err === "object" && (err as { name?: string }).name === "AbortError");
}

function buildStoreFallback(): GitHubReleaseItem[] | null {
  const { releaseNotes, availableVersion, currentVersion, releaseDate } = useUpdateStore.getState();
  if (!releaseNotes) return null;
  const tag = availableVersion ? `v${availableVersion}` : `v${currentVersion}`;
  return [
    {
      tag,
      name: tag,
      publishedAt: releaseDate || "",
      htmlUrl: RELEASES_HTML_URL,
      body: releaseNotes.trim(),
    },
  ];
}

export const releaseService = {
  getCachedReleases(): GitHubReleaseItem[] | null {
    if (!cachedReleases) return null;
    if (Date.now() - cachedReleases.fetchedAt >= RELEASE_CACHE_TTL_MS) return null;
    return cachedReleases.items;
  },

  async fetchAllReleases(options?: { forceRefresh?: boolean; signal?: AbortSignal }): Promise<GitHubReleaseItem[]> {
    const forceRefresh = options?.forceRefresh ?? false;
    const signal = options?.signal;
    if (!forceRefresh) {
      const cached = this.getCachedReleases();
      if (cached) return cached;
    }
    try {
      const res = await fetch(RELEASES_API_URL, {
        headers: { Accept: "application/vnd.github.v3+json" },
        signal,
      });
      if (!res.ok) {
        throw new Error(`GitHub API returned status ${res.status}`);
      }
      const data = await res.json();
      if (!Array.isArray(data)) return [];
      const parsed: GitHubReleaseItem[] = data
        .filter((item) => !item.draft)
        .map((item) => ({
          tag: item.tag_name || "",
          name: item.name || item.tag_name || "",
          publishedAt: item.published_at || "",
          htmlUrl: item.html_url || RELEASES_HTML_URL,
          body: (item.body || "").trim(),
          isPrerelease: !!item.prerelease,
        }));
      cachedReleases = { items: parsed, fetchedAt: Date.now() };
      return parsed;
    } catch (err) {
      if (isAbortError(err)) throw err;
      console.warn("[ReleaseService] Failed to fetch release notes from GitHub API:", err);
      try {
        const resLatest = await fetch(LATEST_JSON_URL, { signal });
        if (resLatest.ok) {
          const latestData = await resLatest.json();
          const fallbackList: GitHubReleaseItem[] = [
            {
              tag: `v${latestData.version}`,
              name: `DesktopBubbleChat v${latestData.version}`,
              publishedAt: latestData.pub_date || "",
              htmlUrl: `${RELEASES_HTML_URL}/latest`,
              body: (latestData.notes || "").trim(),
            },
          ];
          cachedReleases = { items: fallbackList, fetchedAt: Date.now() };
          return fallbackList;
        }
      } catch (latestErr) {
        if (isAbortError(latestErr)) throw latestErr;
        console.warn("[ReleaseService] Failed to fetch latest.json fallback:", latestErr);
      }
      const storeFallback = buildStoreFallback();
      if (storeFallback) return storeFallback;
      throw err;
    }
  },
};
