import React from "react";
import { Activity } from "lucide-react";

interface ConnectionMetricsColumnProps {
  visibleBubbleCount: number;
  totalConnected: number;
  unreadCount: number;
}

export const ConnectionMetricsColumn: React.FC<
  ConnectionMetricsColumnProps
> = ({ visibleBubbleCount, totalConnected, unreadCount }) => {
  return (
    <div className="w-full lg:w-[260px] shrink-0 border-t lg:border-t-0 lg:border-l border-border pt-5 lg:pt-0 lg:pl-6 flex flex-col justify-between gap-3">
      {/* Header */}
      <div className="flex items-center gap-1.5">
        <Activity className="w-3.5 h-3.5 text-primary" />
        <span className="text-caption font-semibold tracking-wider uppercase text-content-secondary">
          Trạng thái
        </span>
      </div>

      {/* Metric 1: Bong bóng bật */}
      <div className="p-3 rounded-card bg-background-subtle border border-border flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-small font-medium text-content-secondary">
            Bong bóng bật
          </span>
          <span className="text-xl font-bold text-content-primary leading-tight">
            {totalConnected > 0
              ? `${visibleBubbleCount}/${totalConnected}`
              : "0"}
          </span>
        </div>
        <div
          className={`w-8 h-8 rounded-panel flex items-center justify-center ${
            visibleBubbleCount > 0
              ? "bg-primary/10 text-primary"
              : "bg-border/60 text-content-muted"
          }`}
        >
          <span className="text-xs font-bold">
            {visibleBubbleCount > 0 ? "ON" : "OFF"}
          </span>
        </div>
      </div>

      {/* Metric 2: Tin chưa đọc */}
      <div className="p-3 rounded-card bg-background-subtle border border-border flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-small font-medium text-content-secondary">
            Tin chưa đọc
          </span>
          <span className="text-xl font-bold text-content-primary leading-tight">
            {unreadCount}
          </span>
        </div>
        <div
          className={`px-2 py-1 rounded-control text-caption font-bold ${
            unreadCount > 0
              ? "bg-status-errorBg text-status-error border border-status-errorBorder/60"
              : "bg-background-app text-content-muted"
          }`}
        >
          {unreadCount > 0 ? `${unreadCount} mới` : "0"}
        </div>
      </div>
    </div>
  );
};

export default ConnectionMetricsColumn;
