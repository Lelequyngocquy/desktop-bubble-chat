import React from "react";
import {
  ArrowDownCircle,
  CheckCircle2,
  Download,
  Info,
  Pause,
  RefreshCw,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { SwitchButton } from "../../../common";
import { formatBytes } from "../../../../hooks/useAutoUpdater";
import { type UpdateStatus } from "../../../../stores/updateStore";
import { type UpdateMode } from "../../../../stores/settingsStore";
import { cn } from "../../../../utils/cn";

export interface UpdateStatusCardProps {
  status: UpdateStatus;
  currentVersion: string;
  availableVersion: string | null;
  releaseDate: string | null;
  downloadedBytes: number;
  totalBytes: number;
  progressPercent: number;
  downloadSpeed: string;
  errorMessage: string | null;
  lastCheckedTime: string | null;
  updateMode: UpdateMode;
  onCheckForUpdates: () => void;
  onStartDownload: () => void;
  onCancelDownload: () => void;
  onOpenConfirmInstall: () => void;
  onSetUpdateMode: (mode: UpdateMode) => void;
}

export const UpdateStatusCard: React.FC<UpdateStatusCardProps> = ({
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
  onCheckForUpdates,
  onStartDownload,
  onCancelDownload,
  onOpenConfirmInstall,
  onSetUpdateMode,
}) => {
  const { t, i18n } = useTranslation("settings");
  const isChecking = status === "checking";
  const isDownloading = status === "downloading";
  const isDownloaded = status === "downloaded";
  const isInstalling = status === "installing";
  const isAvailable = status === "available";
  const isUpToDate = status === "up-to-date";
  const isError = status === "error";
  const getStatusText = (): string => {
    if (isChecking) return t("update.status_checking");
    if (isDownloading) return t("update.status_downloading", { version: availableVersion });
    if (isDownloaded) return t("update.status_downloaded", { version: availableVersion });
    if (isInstalling) return t("update.status_installing");
    if (isError) return t("update.status_error");
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
    if (isInstalling) return t("update.desc_installing");
    if (isError) return errorMessage || t("update.desc_error");
    if (isAvailable) return t("update.desc_available", { version: availableVersion });
    if (isUpToDate) return t("update.desc_uptodate", { version: currentVersion });
    return t("update.desc_ready");
  };
  const formattedReleaseDate = (() => {
    if (!releaseDate) return "";
    const parsed = new Date(releaseDate);
    if (Number.isNaN(parsed.getTime())) return releaseDate;
    return parsed.toLocaleDateString(i18n.language === "vi" ? "vi-VN" : "en-US");
  })();
  return (
    <div className="p-4 bg-background-subtle/70 border border-border rounded-panel flex flex-col gap-4">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-start gap-3.5">
          <div
            className={cn(
              "w-12 h-12 rounded-xl flex items-center justify-center shrink-0 shadow-xs",
              isAvailable || isDownloaded
                ? "bg-primary text-white shadow-primary/20"
                : isUpToDate
                  ? "bg-background-subtle text-status-success border border-border"
                  : isChecking
                    ? "bg-background-subtle text-primary border border-border"
                    : "bg-background-subtle text-content-secondary border border-border",
            )}
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
            {formattedReleaseDate && (
              <span className="text-caption text-content-muted mt-0.5">
                {t("update.release_date")}: {formattedReleaseDate}
              </span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-wrap shrink-0">
          {isAvailable && (
            <button
              type="button"
              onClick={onStartDownload}
              className="flex items-center gap-2 px-3.5 py-1.5 bg-primary hover:bg-primary-foreground text-white rounded-control text-small font-semibold shadow-xs transition-colors cursor-pointer"
            >
              <Download className="w-4 h-4" />
              <span>{t("update.download_btn")}</span>
            </button>
          )}
          {isDownloading && (
            <button
              type="button"
              onClick={onCancelDownload}
              className="flex items-center gap-2 px-3.5 py-1.5 bg-status-errorBg hover:bg-status-errorBg/80 text-status-error border border-status-errorBorder rounded-control text-small font-semibold transition-colors cursor-pointer"
            >
              <Pause className="w-4 h-4" />
              <span>{t("update.pause_btn")}</span>
            </button>
          )}
          {(isDownloaded || isInstalling) && (
            <button
              type="button"
              onClick={onOpenConfirmInstall}
              disabled={isInstalling}
              title={t("update.relaunch_tooltip")}
              className="flex items-center gap-2 px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-control text-small font-semibold shadow-xs transition-colors disabled:opacity-60 animate-pulse cursor-pointer"
            >
              <RotateCcw className={cn("w-4 h-4", isInstalling && "animate-spin")} />
              <span>
                {isInstalling ? t("update.applying_btn") : t("update.relaunch_btn")}
              </span>
            </button>
          )}
          {!isDownloading && !isInstalling && (
            <button
              type="button"
              onClick={onCheckForUpdates}
              disabled={isChecking}
              className={cn(
                "flex items-center gap-2 px-3 py-1.5 border border-border hover:bg-background-subtle bg-background-surface text-content-primary rounded-control text-small font-medium transition-colors cursor-pointer",
                isChecking && "opacity-60 cursor-not-allowed",
              )}
            >
              <RefreshCw
                className={cn(
                  "w-3.5 h-3.5 text-content-secondary",
                  isChecking && "animate-spin text-primary",
                )}
              />
              <span>{isChecking ? t("update.scanning_btn") : t("update.check_btn")}</span>
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
              className={cn(
                "h-full rounded-pill transition-all duration-300",
                isDownloaded ? "bg-emerald-500" : "bg-gradient-to-r from-primary to-blue-400",
              )}
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
            onClick={onCheckForUpdates}
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
          <span
            className={cn(
              "px-2 py-0.5 rounded-pill text-[11px] font-semibold",
              updateMode === "auto"
                ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30"
                : "bg-background-subtle text-content-secondary border border-border",
            )}
          >
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
            onChange={(checked) => onSetUpdateMode(checked ? "auto" : "manual")}
            ariaLabel={t("update.autoupdate_mode_aria")}
          />
        </div>
      </div>
    </div>
  );
};
