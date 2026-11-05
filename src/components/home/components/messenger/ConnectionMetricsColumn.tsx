import React from "react";
import { Activity } from "lucide-react";
import { useTranslation } from "react-i18next";

interface ConnectionMetricsColumnProps {
  visibleBubbleCount: number;
  totalConnected: number;
  unreadCount: number;
}

export const ConnectionMetricsColumn: React.FC<
  ConnectionMetricsColumnProps
> = ({ visibleBubbleCount, totalConnected, unreadCount }) => {
  const { t } = useTranslation("dashboard");

  return (
    <div className="w-full lg:w-[260px] shrink-0 border-t lg:border-t-0 lg:border-l border-border pt-5 lg:pt-0 lg:pl-6 flex flex-col justify-between gap-3">
      <div className="flex items-center gap-1.5">
        <Activity className="w-3.5 h-3.5 text-primary" />
        <span className="text-caption font-semibold tracking-wider uppercase text-content-secondary">
          {t("metrics.status_header")}
        </span>
      </div>
      <div className="p-3 rounded-card bg-background-subtle border border-border flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-small font-medium text-content-secondary">
            {t("metrics.bubbles_active")}
          </span>
          <span className="text-xl font-bold text-content-primary leading-tight">
            {totalConnected > 0
              ? `${visibleBubbleCount}/${totalConnected}`
              : "0"}
          </span>
        </div>
        <div
          className={`w-8 h-8 rounded-panel border flex items-center justify-center ${
            visibleBubbleCount > 0
              ? "bg-background-surface border-border text-primary"
              : "bg-background-surface border-border text-content-muted"
          }`}
        >
          <span className="text-xs font-bold">
            {visibleBubbleCount > 0 ? "ON" : "OFF"}
          </span>
        </div>
      </div>
      <div className="p-3 rounded-card bg-background-subtle border border-border flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-small font-medium text-content-secondary">
            {t("metrics.unread_messages")}
          </span>
          <span className="text-xl font-bold text-content-primary leading-tight">
            {unreadCount}
          </span>
        </div>
        <div
          className={`px-2 py-1 rounded-control text-caption font-bold ${
            unreadCount > 0
              ? "bg-status-errorBg text-status-error border border-status-errorBorder"
              : "bg-background-app text-content-muted"
          }`}
        >
          {unreadCount > 0 ? t("metrics.new_count", { count: unreadCount }) : "0"}
        </div>
      </div>
    </div>
  );
};

export default ConnectionMetricsColumn;