import React, { useEffect, useState, useCallback, useRef } from "react";
import { ArrowDownCircle, ExternalLink, RotateCcw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonDialog, CommonLargeCard } from "../../common";
import { useAutoUpdater } from "../../../hooks/useAutoUpdater";
import { systemService } from "../../../services/systemService";
import { releaseService, type GitHubReleaseItem } from "../../../services/releaseService";
import { UpdateStatusCard } from "./update/UpdateStatusCard";
import { ReleaseHistoryList } from "./update/ReleaseHistoryList";

export const AppUpdateSection: React.FC = () => {
  const { t } = useTranslation(["settings", "common"]);
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
  const isInstalling = status === "installing";
  const hasUpdate = status === "available" || status === "downloading" || status === "downloaded" || isInstalling;
  const [releases, setReleases] = useState<GitHubReleaseItem[]>(() => releaseService.getCachedReleases() || []);
  const [expandedTag, setExpandedTag] = useState<string | null>(() => {
    const cached = releaseService.getCachedReleases();
    return cached && cached.length > 0 ? cached[0].tag : null;
  });
  const [isLoadingNotes, setIsLoadingNotes] = useState(() => !(releaseService.getCachedReleases()?.length));
  const [notesError, setNotesError] = useState<string | null>(null);
  const [isConfirmInstallOpen, setIsConfirmInstallOpen] = useState(false);
  const activeAbortRef = useRef<AbortController | null>(null);
  const loadReleases = useCallback(async (forceRefresh: boolean = false) => {
    activeAbortRef.current?.abort();
    const controller = new AbortController();
    activeAbortRef.current = controller;
    if (forceRefresh) {
      setIsLoadingNotes(true);
    }
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
    const controller = new AbortController();
    activeAbortRef.current = controller;
    releaseService
      .fetchAllReleases({ forceRefresh: false, signal: controller.signal })
      .then((list) => {
        if (controller.signal.aborted) return;
        setReleases(list);
        if (list.length > 0) {
          setExpandedTag((prev) => (prev && list.some((r) => r.tag === prev) ? prev : list[0].tag));
        }
        setNotesError(null);
      })
      .catch((err) => {
        if (controller.signal.aborted) return;
        console.warn("[AppUpdateSection] Release history load failed:", err);
        setNotesError(t("update.notes_error"));
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsLoadingNotes(false);
        }
      });
    return () => {
      controller.abort();
    };
  }, [t]);
  const handleToggleRelease = (tag: string) => {
    setExpandedTag((prev) => (prev === tag ? null : tag));
  };
  const handleOpenGitHubReleases = () => {
    systemService.openExternalUrl(
      "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
    );
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
        <UpdateStatusCard
          status={status}
          currentVersion={currentVersion}
          availableVersion={availableVersion}
          releaseDate={releaseDate}
          downloadedBytes={downloadedBytes}
          totalBytes={totalBytes}
          progressPercent={progressPercent}
          downloadSpeed={downloadSpeed}
          errorMessage={errorMessage}
          lastCheckedTime={lastCheckedTime}
          updateMode={updateMode}
          onCheckForUpdates={() => checkForUpdates(false)}
          onStartDownload={startDownload}
          onCancelDownload={cancelDownload}
          onOpenConfirmInstall={() => setIsConfirmInstallOpen(true)}
          onSetUpdateMode={setUpdateMode}
        />
        <ReleaseHistoryList
          releases={releases}
          currentVersion={currentVersion}
          expandedTag={expandedTag}
          isLoadingNotes={isLoadingNotes}
          notesError={notesError}
          onRefresh={() => loadReleases(true)}
          onToggleRelease={handleToggleRelease}
        />
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
