import React from "react";
import { Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUpdateStore } from "../../../stores/updateStore";
import { cn } from "../../../utils/cn";

export interface SidebarUpdateCardProps {
  onNavigateToUpdate: () => void;
}

export const SidebarUpdateCard: React.FC<SidebarUpdateCardProps> = ({
  onNavigateToUpdate,
}) => {
  const { t } = useTranslation(["common", "settings"]);
  const updateStatus = useUpdateStore((s) => s.status);
  const currentVersion = useUpdateStore((s) => s.currentVersion);
  const availableVersion = useUpdateStore((s) => s.availableVersion);
  const hasUpdate = updateStatus === "available" || updateStatus === "downloading" || updateStatus === "downloaded";
  const statusDescription = hasUpdate
    ? availableVersion
      ? t("common:update_widget_desc_ver", { version: availableVersion })
      : t("common:update_widget_desc")
    : updateStatus === "checking"
      ? t("settings:update.status_checking")
      : currentVersion
        ? t("common:update_widget_uptodate", { version: currentVersion })
        : t("settings:update.status_uptodate");
  return (
    <button
      type="button"
      onClick={onNavigateToUpdate}
      className="w-full text-left bg-sidebar-card border border-sidebar-cardBorder hover:border-primary/50 dark:hover:border-primary/50 rounded-panel p-[14px] flex flex-col gap-2 cursor-pointer transition-all shadow-xs group"
      title={t("settings:scroll_to_update")}
    >
      <span className="w-full flex items-center justify-between text-content-primary">
        <span className="flex items-center gap-2 min-w-0">
          <Sparkles
            className={cn(
              "w-4 h-4 shrink-0",
              hasUpdate ? "text-primary" : "text-content-secondary group-hover:text-primary transition-colors",
            )}
          />
          <span className="text-small font-medium leading-none truncate">
            {t("common:update_widget_title")}
          </span>
        </span>
        {hasUpdate && (
          <span className="px-1.5 py-0.5 bg-status-error text-content-inverse text-[10px] font-bold rounded-pill animate-pulse leading-none shadow-xs shrink-0">
            {t("common:new_badge_tag")}
          </span>
        )}
      </span>
      <span className="text-caption text-content-secondary leading-snug group-hover:text-content-primary transition-colors block">
        {statusDescription}
      </span>
    </button>
  );
};
