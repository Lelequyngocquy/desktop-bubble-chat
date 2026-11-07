import React, { useEffect, useState, useCallback } from "react";
import {
  ArrowDownCircle,
  CheckCircle2,
  ChevronDown,
  Download,
  ExternalLink,
  Info,
  Pause,
  RefreshCw,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonLargeCard, SwitchButton } from "../../common";
import { formatBytes, useAutoUpdater } from "../../../hooks/useAutoUpdater";
import { systemService } from "../../../services/systemService";

interface GitHubReleaseItem {
  tag: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  body: string;
  isPrerelease?: boolean;
}

export const AppUpdateSection: React.FC = () => {
  const { t, i18n } = useTranslation("settings");
  const {
    status,
    currentVersion,
    availableVersion,
    releaseDate,
    releaseNotes,
    downloadedBytes,
    totalBytes,
    progressPercent,
    downloadSpeed,
    errorMessage,
    lastCheckedTime,
    autoCheckOnStartup,
    checkForUpdates,
    startDownload,
    cancelDownload,
    installAndRelaunch,
    setAutoCheckOnStartup,
  } = useAutoUpdater();
  const isChecking = status === "checking";
  const isDownloading = status === "downloading";
  const isDownloaded = status === "downloaded";
  const isInstalling = status === "installing";
  const isAvailable = status === "available";
  const isUpToDate = status === "up-to-date";
  const hasUpdate = isAvailable || isDownloading || isDownloaded;
  const [releases, setReleases] = useState<GitHubReleaseItem[]>([]);
  const [expandedTag, setExpandedTag] = useState<string | null>(null);
  const [isLoadingNotes, setIsLoadingNotes] = useState(false);
  const [notesError, setNotesError] = useState<string | null>(null);
  const fetchAllGitHubReleases = useCallback(async () => {
    setIsLoadingNotes(true);
    setNotesError(null);
    try {
      const res = await fetch(
        "https://api.github.com/repos/Lelequyngocquy/desktop-bubble-chat/releases?per_page=50",
        {
          headers: {
            Accept: "application/vnd.github.v3+json",
          },
        },
      );
      if (!res.ok) {
        throw new Error(`GitHub API returned status ${res.status}`);
      }
      const data = await res.json();
      if (Array.isArray(data)) {
        const parsed: GitHubReleaseItem[] = data
          .filter((item) => !item.draft)
          .map((item) => ({
            tag: item.tag_name || "",
            name: item.name || item.tag_name || "",
            publishedAt: item.published_at || "",
            htmlUrl:
              item.html_url ||
              "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
            body: (item.body || "").trim(),
            isPrerelease: !!item.prerelease,
          }));
        setReleases(parsed);
        if (parsed.length > 0) {
          setExpandedTag((prev) =>
            prev && parsed.some((r) => r.tag === prev) ? prev : parsed[0].tag,
          );
        }
      }
    } catch (err) {
      console.warn(
        "[AppUpdateSection] Failed to fetch release notes from GitHub API:",
        err,
      );
      try {
        const resLatest = await fetch(
          "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest/download/latest.json",
        );
        if (resLatest.ok) {
          const latestData = await resLatest.json();
          const fallbackList: GitHubReleaseItem[] = [
            {
              tag: `v${latestData.version}`,
              name: `DesktopBubbleChat v${latestData.version}`,
              publishedAt: latestData.pub_date || "",
              htmlUrl:
                "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest",
              body: (latestData.notes || "").trim(),
            },
          ];
          setReleases(fallbackList);
          setExpandedTag(fallbackList[0].tag);
          return;
        }
      } catch {}
      if (releaseNotes) {
        const fallbackList: GitHubReleaseItem[] = [
          {
            tag: availableVersion ? `v${availableVersion}` : `v${currentVersion}`,
            name: availableVersion ? `v${availableVersion}` : `v${currentVersion}`,
            publishedAt: releaseDate || "",
            htmlUrl:
              "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
            body: releaseNotes.trim(),
          },
        ];
        setReleases(fallbackList);
        setExpandedTag(fallbackList[0].tag);
      } else {
        setNotesError(t("update.notes_error"));
      }
    } finally {
      setIsLoadingNotes(false);
    }
  }, [availableVersion, currentVersion, releaseDate, releaseNotes, t]);
  useEffect(() => {
    fetchAllGitHubReleases();
  }, [fetchAllGitHubReleases]);
  const handleToggleRelease = (tag: string) => {
    setExpandedTag((prev) => (prev === tag ? null : tag));
  };
  const handleOpenGitHubReleases = () => {
    systemService.openExternalUrl(
      "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
    );
  };
  const formatReleaseDate = (isoString?: string): string => {
    if (!isoString) return "";
    try {
      return new Date(isoString).toLocaleDateString(
        i18n.language === "vi" ? "vi-VN" : "en-US",
        { year: "numeric", month: "short", day: "numeric" },
      );
    } catch {
      return isoString;
    }
  };
  const getStatusText = (): string => {
    if (isChecking) return t("update.status_checking");
    if (isDownloading)
      return t("update.status_downloading", { version: availableVersion });
    if (isDownloaded)
      return t("update.status_downloaded", { version: availableVersion });
    if (isInstalling) return t("update.status_installing");
    if (isAvailable)
      return t("update.status_available", { version: availableVersion });
    if (isUpToDate) return t("update.status_uptodate");
    return t("update.status_ready");
  };
  const getDescriptionText = (): string => {
    if (isChecking) return t("update.desc_checking");
    if (isDownloading) {
      return t("update.desc_downloading", {
        percent: progressPercent,
        downloaded: formatBytes(downloadedBytes),
        total: formatBytes(totalBytes),
        speed: downloadSpeed,
      });
    }
    if (isDownloaded) return t("update.desc_downloaded");
    if (isAvailable)
      return t("update.desc_available", { version: availableVersion });
    if (isUpToDate)
      return t("update.desc_uptodate", { version: currentVersion });
    return t("update.desc_ready");
  };
  return (
    <CommonLargeCard
      icon={<ArrowDownCircle className="w-4 h-4 text-primary" />}
      title={t("update.title")}
      subTitle={
        hasUpdate ? (
          <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-primary text-white shadow-xs">
            {t("update.new_version_badge", { version: availableVersion })}
          </span>
        ) : (
          <span className="text-caption text-content-secondary">
            v{currentVersion}
          </span>
        )
      }
      linkTitle={
        <button
          onClick={handleOpenGitHubReleases}
          className="flex items-center gap-1.5 text-caption text-primary hover:underline font-medium transition-colors cursor-pointer"
          title={t("update.github_releases_tooltip")}
        >
          <span>{t("update.github_releases")}</span>
          <ExternalLink className="w-3.5 h-3.5" />
        </button>
      }
    >
      <div className="flex flex-col gap-5">
        <div className="p-4 bg-background-subtle/70 border border-border rounded-panel flex flex-col gap-4">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="flex items-start gap-3.5">
              <div
                className={`w-12 h-12 rounded-xl flex items-center justify-center shrink-0 shadow-xs ${
                  isAvailable || isDownloaded
                    ? "bg-primary text-white shadow-primary/20"
                    : isUpToDate
                      ? "bg-background-subtle text-status-success border border-border"
                      : isChecking
                        ? "bg-background-subtle text-primary border border-border"
                        : "bg-background-subtle text-content-secondary border border-border"
                }`}
              >
                {isChecking ? (
                  <RefreshCw className="w-6 h-6 animate-spin" />
                ) : isAvailable ? (
                  <Sparkles className="w-6 h-6 animate-bounce" />
                ) : isDownloaded ? (
                  <Download className="w-6 h-6 text-white" />
                ) : isUpToDate ? (
                  <CheckCircle2 className="w-6 h-6" />
                ) : (
                  <ArrowDownCircle className="w-6 h-6" />
                )}
              </div>
              <div className="flex flex-col gap-0.5">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-body font-semibold text-content-primary">
                    {getStatusText()}
                  </span>
                  {isAvailable && (
                    <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-primary text-white shadow-xs">
                      {t("update.new_badge")}
                    </span>
                  )}
                  {isDownloaded && (
                    <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-emerald-600 text-white shadow-xs">
                      {t("update.ready_to_activate")}
                    </span>
                  )}
                </div>
                <p className="text-small text-content-secondary m-0">
                  {getDescriptionText()}
                </p>
                {releaseDate && (
                  <span className="text-caption text-content-muted mt-0.5">
                    {t("update.release_date")}:{" "}
                    {new Date(releaseDate).toLocaleDateString(
                      i18n.language === "vi" ? "vi-VN" : "en-US",
                    )}
                  </span>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2 flex-wrap shrink-0">
              {isAvailable && (
                <button
                  onClick={startDownload}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-primary hover:bg-primary-hover text-white rounded-control text-small font-semibold shadow-xs transition-colors cursor-pointer"
                >
                  <Download className="w-4 h-4" />
                  <span>{t("update.download_btn")}</span>
                </button>
              )}
              {isDownloading && (
                <button
                  onClick={cancelDownload}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-rose-50 hover:bg-rose-100 text-rose-600 border border-rose-200 rounded-control text-small font-semibold transition-colors cursor-pointer"
                >
                  <Pause className="w-4 h-4" />
                  <span>{t("update.pause_btn")}</span>
                </button>
              )}
              {isDownloaded && (
                <button
                  onClick={installAndRelaunch}
                  disabled={isInstalling}
                  title={t("update.relaunch_tooltip")}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-control text-small font-semibold shadow-xs transition-colors disabled:opacity-60 animate-pulse cursor-pointer"
                >
                  <RotateCcw
                    className={`w-4 h-4 ${isInstalling ? "animate-spin" : ""}`}
                  />
                  <span>
                    {isInstalling
                      ? t("update.applying_btn")
                      : t("update.relaunch_btn")}
                  </span>
                </button>
              )}
              {!isDownloading && !isInstalling && (
                <button
                  onClick={() => checkForUpdates(false)}
                  disabled={isChecking}
                  className={`flex items-center gap-2 px-3 py-1.5 border border-border hover:bg-background-subtle bg-background-surface text-content-primary rounded-control text-small font-medium transition-colors cursor-pointer ${
                    isChecking ? "opacity-60 cursor-not-allowed" : ""
                  }`}
                >
                  <RefreshCw
                    className={`w-3.5 h-3.5 text-content-secondary ${
                      isChecking ? "animate-spin text-primary" : ""
                    }`}
                  />
                  <span>
                    {isChecking
                      ? t("update.scanning_btn")
                      : t("update.check_btn")}
                  </span>
                </button>
              )}
            </div>
          </div>
          {(isDownloading || isDownloaded) && (
            <div className="flex flex-col gap-1.5 pt-2 border-t border-border">
              <div className="flex items-center justify-between text-caption font-medium">
                <span className="text-content-secondary">
                  {isDownloaded
                    ? t("update.download_complete")
                    : `${t("update.downloading", { percent: progressPercent })} · ${downloadSpeed}`}
                </span>
                <span className="text-content-primary font-semibold">
                  {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
                </span>
              </div>
              <div className="w-full h-2.5 bg-slate-100 dark:bg-background-app rounded-pill overflow-hidden border border-border p-0.5">
                <div
                  className={`h-full rounded-pill transition-all duration-300 ${
                    isDownloaded
                      ? "bg-emerald-500"
                      : "bg-gradient-to-r from-primary to-blue-400"
                  }`}
                  style={{ width: `${Math.max(4, progressPercent)}%` }}
                />
              </div>
            </div>
          )}
          {errorMessage && (
            <div className="p-3 bg-rose-50 border border-rose-200 rounded-control text-rose-700 text-small flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Info className="w-4 h-4 shrink-0 text-rose-500" />
                <span>{errorMessage}</span>
              </div>
              <button
                onClick={() => checkForUpdates(false)}
                className="text-xs font-semibold text-rose-700 underline hover:no-underline shrink-0 cursor-pointer"
              >
                {t("update.retry")}
              </button>
            </div>
          )}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2 border-t border-border text-small">
            <div className="flex items-center gap-2 text-content-secondary">
              <span className="w-1.5 h-1.5 rounded-full bg-content-muted" />
              <span>
                {t("update.last_checked")}:{" "}
                <strong className="text-content-primary font-medium">
                  {lastCheckedTime || t("update.never_checked")}
                </strong>
              </span>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-content-secondary">
                {t("update.autocheck_label")}
              </span>
              <SwitchButton
                checked={autoCheckOnStartup}
                onChange={setAutoCheckOnStartup}
                ariaLabel={t("update.autocheck_aria")}
              />
            </div>
          </div>
        </div>
        <div className="flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <h3 className="text-body font-semibold text-content-primary m-0">
                {t("update.release_history")}
              </h3>
              {releases.length > 0 && (
                <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-background-subtle border border-border text-content-secondary">
                  {t("update.all_releases_count", { count: releases.length })}
                </span>
              )}
            </div>
            <button
              onClick={fetchAllGitHubReleases}
              disabled={isLoadingNotes}
              className="flex items-center gap-1.5 text-caption text-content-secondary hover:text-primary transition-colors cursor-pointer"
              title={t("update.refresh_notes_tooltip")}
            >
              <RefreshCw
                className={`w-3.5 h-3.5 ${isLoadingNotes ? "animate-spin text-primary" : ""}`}
              />
              <span>
                {isLoadingNotes
                  ? t("update.refreshing_notes")
                  : t("update.refresh_notes")}
              </span>
            </button>
          </div>
          {isLoadingNotes && releases.length === 0 ? (
            <div className="flex items-center justify-center gap-2.5 py-8 text-content-secondary border border-border rounded-control bg-background-app/50">
              <RefreshCw className="w-4 h-4 animate-spin text-primary" />
              <span className="text-small">{t("update.loading_notes")}</span>
            </div>
          ) : notesError && releases.length === 0 ? (
            <div className="flex flex-col items-center justify-center gap-2 py-6 border border-border rounded-control bg-background-app/50 text-content-muted">
              <span className="text-small">{notesError}</span>
              <button
                onClick={fetchAllGitHubReleases}
                className="text-xs text-primary underline hover:no-underline font-medium cursor-pointer"
              >
                {t("update.retry")}
              </button>
            </div>
          ) : releases.length === 0 ? (
            <div className="border border-border rounded-control p-6 bg-background-app/50 text-content-muted text-center text-small">
              {t("update.no_notes")}
            </div>
          ) : (
            <div className="border border-border rounded-control overflow-hidden divide-y divide-border bg-background-surface">
              {releases.map((rel, idx) => {
                const isExpanded = expandedTag === rel.tag;
                const isLatest = idx === 0;
                const isCurrent =
                  rel.tag.replace(/^v/, "") === currentVersion ||
                  rel.tag === currentVersion;
                return (
                  <div key={rel.tag || idx} className="flex flex-col">
                    <button
                      type="button"
                      onClick={() => handleToggleRelease(rel.tag)}
                      aria-expanded={isExpanded}
                      className={`w-full flex items-center justify-between gap-3 px-4 py-3 text-left transition-colors cursor-pointer ${
                        isExpanded
                          ? "bg-background-subtle/80"
                          : "hover:bg-background-subtle/40 bg-transparent"
                      }`}
                      title={t("update.collapse_tip")}
                    >
                      <div className="flex items-center gap-2 min-w-0 flex-wrap">
                        <span className="text-small font-semibold text-content-primary">
                          {rel.tag}
                        </span>
                        {isLatest && (
                          <span className="px-1.5 py-0.5 rounded-pill text-[10px] font-semibold bg-primary text-white shadow-xs">
                            {t("update.latest_badge")}
                          </span>
                        )}
                        {isCurrent && (
                          <span className="px-1.5 py-0.5 rounded-pill text-[10px] font-semibold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30">
                            {t("update.current_badge")}
                          </span>
                        )}
                        {rel.name && rel.name !== rel.tag && (
                          <span className="text-caption text-content-secondary truncate max-w-[200px] sm:max-w-[320px]">
                            {rel.name}
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        {rel.publishedAt && (
                          <span className="text-caption text-content-muted hidden sm:inline">
                            {formatReleaseDate(rel.publishedAt)}
                          </span>
                        )}
                        <span
                          role="button"
                          tabIndex={0}
                          onClick={(e) => {
                            e.stopPropagation();
                            systemService.openExternalUrl(rel.htmlUrl);
                          }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                              e.stopPropagation();
                              systemService.openExternalUrl(rel.htmlUrl);
                            }
                          }}
                          title={t("update.open_in_github")}
                          className="p-1 rounded-md text-content-muted hover:text-primary hover:bg-background-app transition-colors"
                        >
                          <ExternalLink className="w-3.5 h-3.5" />
                        </span>
                        <ChevronDown
                          className={`w-4 h-4 text-content-muted transition-transform duration-200 ${
                            isExpanded ? "rotate-180 text-primary" : ""
                          }`}
                        />
                      </div>
                    </button>
                    {isExpanded && (
                      <div className="px-4 py-3.5 bg-background-app/40 border-t border-border">
                        <div className="text-small text-content-primary leading-relaxed whitespace-pre-wrap font-sans select-text max-h-[380px] overflow-y-auto custom-scrollbar">
                          {rel.body || (
                            <span className="text-content-muted italic">
                              {t("update.no_notes")}
                            </span>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default AppUpdateSection;
