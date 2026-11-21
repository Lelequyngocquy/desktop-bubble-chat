import React from "react";
import { ChevronDown, ExternalLink, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { systemService } from "../../../../services/systemService";
import { type GitHubReleaseItem } from "../../../../services/releaseService";
import { cn } from "../../../../utils/cn";

export interface ReleaseHistoryListProps {
  releases: GitHubReleaseItem[];
  currentVersion: string;
  expandedTag: string | null;
  isLoadingNotes: boolean;
  notesError: string | null;
  onRefresh: () => void;
  onToggleRelease: (tag: string) => void;
}

export const ReleaseHistoryList: React.FC<ReleaseHistoryListProps> = ({
  releases,
  currentVersion,
  expandedTag,
  isLoadingNotes,
  notesError,
  onRefresh,
  onToggleRelease,
}) => {
  const { t, i18n } = useTranslation("settings");
  const formatReleaseDate = (isoString?: string): string => {
    if (!isoString) return "";
    const parsed = new Date(isoString);
    if (Number.isNaN(parsed.getTime())) return isoString;
    return parsed.toLocaleDateString(
      i18n.language === "vi" ? "vi-VN" : "en-US",
      { year: "numeric", month: "short", day: "numeric" },
    );
  };
  return (
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
          onClick={onRefresh}
          disabled={isLoadingNotes}
          className="flex items-center gap-1.5 text-caption text-content-secondary hover:text-primary transition-colors cursor-pointer"
          title={t("update.refresh_notes_tooltip")}
        >
          <RefreshCw
            className={cn("w-3.5 h-3.5", isLoadingNotes && "animate-spin text-primary")}
          />
          <span>
            {isLoadingNotes ? t("update.refreshing_notes") : t("update.refresh_notes")}
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
            onClick={onRefresh}
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
            const isCurrent = rel.tag.replace(/^v/, "") === currentVersion || rel.tag === currentVersion;
            return (
              <div key={rel.tag || idx} className="flex flex-col">
                <div
                  className={cn(
                    "w-full flex items-center justify-between gap-3 px-4 py-3 transition-colors",
                    isExpanded ? "bg-background-subtle/80" : "hover:bg-background-subtle/40 bg-transparent",
                  )}
                >
                  <button
                    type="button"
                    onClick={() => onToggleRelease(rel.tag)}
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
                      onClick={() => onToggleRelease(rel.tag)}
                      aria-expanded={isExpanded}
                      aria-label={t("update.collapse_tip")}
                      className="p-1 rounded-md text-content-muted hover:text-primary transition-colors cursor-pointer"
                    >
                      <ChevronDown
                        className={cn(
                          "w-4 h-4 transition-transform duration-200",
                          isExpanded && "rotate-180 text-primary",
                        )}
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
  );
};
