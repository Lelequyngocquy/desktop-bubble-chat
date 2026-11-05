import React, { useEffect, useState, useCallback } from "react";
import {
  ArrowUpCircle,
  CheckCircle2,
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

interface GitHubReleaseData {
  tag: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  body: string;
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
  const [githubRelease, setGithubRelease] = useState<GitHubReleaseData | null>(null);
  const [isLoadingNotes, setIsLoadingNotes] = useState(false);
  const [notesError, setNotesError] = useState<string | null>(null);

  const fetchGitHubReleaseNotes = useCallback(async () => {
    setIsLoadingNotes(true);
    setNotesError(null);
    try {
      const res = await fetch(
        "https://api.github.com/repos/Lelequyngocquy/desktop-bubble-chat/releases/latest",
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
      setGithubRelease({
        tag: data.tag_name || `v${availableVersion || currentVersion}`,
        name: data.name || "",
        publishedAt: data.published_at || "",
        htmlUrl:
          data.html_url ||
          "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
        body: (data.body || "").trim(),
      });
    } catch (err) {
      console.warn("[AppUpdateSection] Failed to fetch release notes from GitHub API:", err);
      try {
        const resLatest = await fetch(
          "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest/download/latest.json",
        );
        if (resLatest.ok) {
          const latestData = await resLatest.json();
          setGithubRelease({
            tag: `v${latestData.version}`,
            name: `DesktopBubbleChat v${latestData.version}`,
            publishedAt: latestData.pub_date || "",
            htmlUrl:
              "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest",
            body: (latestData.notes || "").trim(),
          });
          return;
        }
      } catch {}
      if (releaseNotes) {
        setGithubRelease({
          tag: availableVersion ? `v${availableVersion}` : `v${currentVersion}`,
          name: "",
          publishedAt: releaseDate || "",
          htmlUrl:
            "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
          body: releaseNotes.trim(),
        });
      } else {
        setNotesError(t("update.notes_error"));
      }
    } finally {
      setIsLoadingNotes(false);
    }
  }, [availableVersion, currentVersion, releaseDate, releaseNotes, t]);

  useEffect(() => {
    fetchGitHubReleaseNotes();
  }, [fetchGitHubReleaseNotes]);

  const handleOpenGitHubReleases = () => {
    systemService.openExternalUrl(
      githubRelease?.htmlUrl ||
        "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
    );
  };

  const getStatusText = (): string => {
    if (isChecking) return t("update.status_checking");
    if (isDownloading) return t("update.status_downloading", { version: availableVersion });
    if (isDownloaded) return t("update.status_downloaded", { version: availableVersion });
    if (isInstalling) return t("update.status_installing");
    if (isAvailable) return t("update.status_available", { version: availableVersion });
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
    if (isAvailable) return t("update.desc_available", { version: availableVersion });
    if (isUpToDate) return t("update.desc_uptodate", { version: currentVersion });
    return t("update.desc_ready");
  };

  return (
    <CommonLargeCard
      icon={<ArrowUpCircle className="w-4 h-4 text-primary" />}
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
        <div className="p-4 bg-background-subtle/70 border border-border/80 rounded-panel flex flex-col gap-4">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="flex items-start gap-3.5">
              <div
                className={`w-12 h-12 rounded-xl flex items-center justify-center shrink-0 shadow-xs ${
                  isAvailable || isDownloaded
                    ? "bg-primary text-white shadow-primary/20"
                    : isUpToDate
                      ? "bg-emerald-50 text-emerald-600 border border-emerald-200"
                      : isChecking
                        ? "bg-blue-50 text-primary border border-blue-200"
                        : "bg-slate-100 text-content-secondary border border-border"
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
                  <ArrowUpCircle className="w-6 h-6" />
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
                    {new Date(releaseDate).toLocaleDateString(i18n.language === "vi" ? "vi-VN" : "en-US")}
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
                  className={`flex items-center gap-2 px-3 py-1.5 border border-border hover:bg-white bg-background-surface text-content-primary rounded-control text-small font-medium transition-colors cursor-pointer ${
                    isChecking ? "opacity-60 cursor-not-allowed" : ""
                  }`}
                >
                  <RefreshCw
                    className={`w-3.5 h-3.5 text-content-secondary ${
                      isChecking ? "animate-spin text-primary" : ""
                    }`}
                  />
                  <span>
                    {isChecking ? t("update.scanning_btn") : t("update.check_btn")}
                  </span>
                </button>
              )}
            </div>
          </div>
          {(isDownloading || isDownloaded) && (
            <div className="flex flex-col gap-1.5 pt-2 border-t border-border/60">
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
              <div className="w-full h-2.5 bg-slate-100 rounded-pill overflow-hidden border border-border/40 p-0.5">
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
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2 border-t border-border/60 text-small">
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
                {githubRelease?.tag
                  ? t("update.release_notes_new", { tag: githubRelease.tag })
                  : availableVersion
                    ? t("update.release_notes_new", { tag: `v${availableVersion}` })
                    : t("update.release_notes_current", { version: currentVersion })}
              </h3>
            </div>
            <button
              onClick={fetchGitHubReleaseNotes}
              disabled={isLoadingNotes}
              className="flex items-center gap-1.5 text-caption text-content-secondary hover:text-primary transition-colors cursor-pointer"
              title={t("update.refresh_notes_tooltip")}
            >
              <RefreshCw
                className={`w-3.5 h-3.5 ${isLoadingNotes ? "animate-spin text-primary" : ""}`}
              />
              <span>
                {isLoadingNotes ? t("update.refreshing_notes") : t("update.refresh_notes")}
              </span>
            </button>
          </div>
          <div className="bg-background-app/70 border border-border/80 rounded-control p-4 text-small text-content-primary leading-relaxed whitespace-pre-wrap font-sans min-h-[90px]">
            {isLoadingNotes ? (
              <div className="flex items-center justify-center gap-2.5 py-6 text-content-secondary">
                <RefreshCw className="w-4 h-4 animate-spin text-primary" />
                <span>
                  {t("update.loading_notes")}
                </span>
              </div>
            ) : notesError ? (
              <div className="flex flex-col items-center justify-center gap-2 py-4 text-content-muted">
                <span>{notesError}</span>
                <button
                  onClick={fetchGitHubReleaseNotes}
                  className="text-xs text-primary underline hover:no-underline font-medium cursor-pointer"
                >
                  {t("update.retry")}
                </button>
              </div>
            ) : githubRelease?.body ? (
              githubRelease.body
            ) : githubRelease ? (
              <div className="text-content-secondary">
                <p className="m-0">
                  Release <strong className="text-content-primary">{githubRelease.tag}</strong>
                  {githubRelease.name ? ` (${githubRelease.name})` : ""} is ready on GitHub
                  {githubRelease.publishedAt
                    ? ` (${t("update.release_date")}: ${new Date(githubRelease.publishedAt).toLocaleDateString(i18n.language === "vi" ? "vi-VN" : "en-US")})`
                    : ""}
                  .
                </p>
              </div>
            ) : (
              <div className="text-content-muted text-center py-4">
                {t("update.no_notes")}
              </div>
            )}
          </div>
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default AppUpdateSection;