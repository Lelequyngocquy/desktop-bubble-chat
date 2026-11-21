import React, { useEffect, useState, useCallback, useRef } from "react";
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
import { CommonDialog, CommonLargeCard, SwitchButton } from "../../common";
import { formatBytes, useAutoUpdater } from "../../../hooks/useAutoUpdater";
import { systemService } from "../../../services/systemService";
import { releaseService, type GitHubReleaseItem } from "../../../services/releaseService";

export const AppUpdateSection: React.FC = () => {
  const { t, i18n } = useTranslation(["settings", "common"]);
  const {
    status,
    currentVersion,
    availableVersion,
    releaseDate,
    downloadedBytes,
    totalBytes,
    progressPercent,
    downloadSpeed,
    errorMessage,
    lastCheckedTime,
    updateMode,
    checkForUpdates,
    startDownload,
    cancelDownload,
    installAndRelaunch,
    setUpdateMode,
  } = useAutoUpdater();
  const isChecking = status === "checking";
  const isDownloading = status === "downloading";
  const isDownloaded = status === "downloaded";
  const isInstalling = status === "installing";
  const isAvailable = status === "available";
  const isUpToDate = status === "up-to-date";
  const isError = status === "error";
  const hasUpdate = isAvailable || isDownloading || isDownloaded || isInstalling;
  const [releases, setReleases] = useState<GitHubReleaseItem[]>(() => releaseService.getCachedReleases() || []);
  const [expandedTag, setExpandedTag] = useState<string | null>(() => {
    const cached = releaseService.getCachedReleases();
    return cached && cached.length > 0 ? cached[0].tag : null;
  });
  const [isLoadingNotes, setIsLoadingNotes] = useState(false);
  const [notesError, setNotesError] = useState<string | null>(null);
  const [isConfirmInstallOpen, setIsConfirmInstallOpen] = useState(false);
  const activeAbortRef = useRef<AbortController | null>(null);
  const loadReleases = useCallback(async (forceRefresh: boolean = false) => {
    activeAbortRef.current?.abort();
    const controller = new AbortController();
    activeAbortRef.current = controller;
    setIsLoadingNotes(true);
    setNotesError(null);
    try {
      const list = await releaseService.fetchAllReleases({
        forceRefresh,
        signal: controller.signal,
      });
      if (controller.signal.aborted) return;
      setReleases(list);
      if (list.length > 0) {
        setExpandedTag((prev) =>
          prev && list.some((r) => r.tag === prev) ? prev : list[0].tag,
        );
      }
    } catch (err) {
      if (controller.signal.aborted) return;
      console.warn("[AppUpdateSection] Release history load failed:", err);
      setNotesError(t("update.notes_error"));
    } finally {
      if (!controller.signal.aborted) {
        setIsLoadingNotes(false);
      }
    }
  }, [t]);
  useEffect(() => {
    loadReleases(false);
    return () => {
      activeAbortRef.current?.abort();
    };
  }, []);
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
    const parsed = new Date(isoString);
    if (Number.isNaN(parsed.getTime())) return isoString;
    return parsed.toLocaleDateString(
      i18n.language === "vi" ? "vi-VN" : "en-US",
      { year: "numeric", month: "short", day: "numeric" },
    );
  };
  const getStatusText = (): string => {
    if (isChecking) return t("update.status_checking");
    if (isDownloading)
      return t("update.status_downloading", { version: availableVersion });
    if (isDownloaded)
      return t("update.status_downloaded", { version: availableVersion });
    if (isInstalling) return t("update.status_installing");
    if (isError) return t("update.status_error");
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
    if (isInstalling) return t("update.desc_installing");
    if (isError) return errorMessage || t("update.desc_error");
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
          type="button"
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
                  type="button"
                  onClick={startDownload}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-primary hover:bg-primary-foreground text-white rounded-control text-small font-semibold shadow-xs transition-colors cursor-pointer"
                >
                  <Download className="w-4 h-4" />
                  <span>{t("update.download_btn")}</span>
                </button>
              )}
              {isDownloading && (
                <button
                  type="button"
                  onClick={cancelDownload}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-status-errorBg hover:bg-status-errorBg/80 text-status-error border border-status-errorBorder rounded-control text-small font-semibold transition-colors cursor-pointer"
                >
                  <Pause className="w-4 h-4" />
                  <span>{t("update.pause_btn")}</span>
                </button>
              )}
              {(isDownloaded || isInstalling) && (
                <button
                  type="button"
                  onClick={() => setIsConfirmInstallOpen(true)}
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
                  type="button"
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
              <div className="w-full h-2.5 bg-background-app rounded-pill overflow-hidden border border-border p-0.5">
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
            <div className="p-3 bg-status-errorBg border border-status-errorBorder rounded-control text-status-error text-small flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Info className="w-4 h-4 shrink-0 text-status-error" />
                <span>{errorMessage}</span>
              </div>
              <button
                type="button"
                onClick={() => checkForUpdates(false)}
                className="text-xs font-semibold text-status-error underline hover:no-underline shrink-0 cursor-pointer"
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
            <div className="flex items-center gap-2">
              <span className={`px-2 py-0.5 rounded-pill text-[11px] font-semibold ${
                updateMode === "auto"
                  ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30"
                  : "bg-background-subtle text-content-secondary border border-border"
              }`}>
                {updateMode === "auto" ? t("update.mode_auto_badge") : t("update.mode_manual_badge")}
              </span>
            </div>
          </div>
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2 border-t border-border">
            <div className="flex flex-col gap-0.5">
              <span className="text-body font-medium text-content-primary">
                {t("update.autoupdate_mode_title")}
              </span>
              <span className="text-caption text-content-secondary">
                {updateMode === "auto" ? t("update.mode_auto_desc") : t("update.mode_manual_desc")}
              </span>
            </div>
            <div className="flex items-center gap-3 shrink-0">
              <SwitchButton
                checked={updateMode === "auto"}
                onChange={(checked) => setUpdateMode(checked ? "auto" : "manual")}
                ariaLabel={t("update.autoupdate_mode_aria")}
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
              type="button"
              onClick={() => loadReleases(true)}
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
                type="button"
                onClick={() => loadReleases(true)}
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
                    <div
                      className={`w-full flex items-center justify-between gap-3 px-4 py-3 transition-colors ${
                        isExpanded
                          ? "bg-background-subtle/80"
                          : "hover:bg-background-subtle/40 bg-transparent"
                      }`}
                    >
                      <button
                        type="button"
                        onClick={() => handleToggleRelease(rel.tag)}
                        aria-expanded={isExpanded}
                        title={t("update.collapse_tip")}
                        className="flex items-center justify-between gap-3 min-w-0 flex-1 text-left cursor-pointer"
                      >
                        <span className="flex items-center gap-2 min-w-0 flex-wrap">
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
                        </span>
                        {rel.publishedAt && (
                          <span className="text-caption text-content-muted hidden sm:inline shrink-0">
                            {formatReleaseDate(rel.publishedAt)}
                          </span>
                        )}
                      </button>
                      <div className="flex items-center gap-1 shrink-0">
                        <button
                          type="button"
                          onClick={() => systemService.openExternalUrl(rel.htmlUrl)}
                          title={t("update.open_in_github")}
                          aria-label={t("update.open_in_github")}
                          className="p-1 rounded-md text-content-muted hover:text-primary hover:bg-background-app transition-colors cursor-pointer"
                        >
                          <ExternalLink className="w-3.5 h-3.5" />
                        </button>
                        <button
                          type="button"
                          onClick={() => handleToggleRelease(rel.tag)}
                          aria-expanded={isExpanded}
                          aria-label={t("update.collapse_tip")}
                          className="p-1 rounded-md text-content-muted hover:text-primary transition-colors cursor-pointer"
                        >
                          <ChevronDown
                            className={`w-4 h-4 transition-transform duration-200 ${
                              isExpanded ? "rotate-180 text-primary" : ""
                            }`}
                          />
                        </button>
                      </div>
                    </div>
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
      <CommonDialog
        isOpen={isConfirmInstallOpen}
        onClose={() => {
          if (!isInstalling) setIsConfirmInstallOpen(false);
        }}
        closeOnBackdropClick={!isInstalling}
        closeOnEscape={!isInstalling}
        showCloseButton={!isInstalling}
        size="sm"
        eyebrow={t("common:confirm").toUpperCase()}
        titleIcon={<RotateCcw className="w-5 h-5 text-primary" />}
        title={t("update.confirm_install_title")}
        content={t("update.confirm_install_desc", { version: availableVersion || currentVersion })}
        backButton={{
          label: t("common:cancel"),
          disabled: isInstalling,
          onClick: () => setIsConfirmInstallOpen(false),
        }}
        rightButton={{
          label: isInstalling ? t("update.applying_btn") : t("update.confirm_install_btn"),
          variant: "primary",
          loading: isInstalling,
          onClick: async () => {
            setIsConfirmInstallOpen(false);
            await installAndRelaunch();
          },
        }}
      />
    </CommonLargeCard>
  );
};

export default AppUpdateSection;
