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
    <div className="w-full lg:w-[260px] shrink-0 border-t lg:border-t-0 lg:border-l border-[#E2E7EF] pt-5 lg:pt-0 lg:pl-6 flex flex-col justify-between gap-3">
      {/* Header */}
      <div className="flex items-center gap-1.5">
        <Activity className="w-3.5 h-3.5 text-[#0084FF]" />
        <span className="text-[11px] font-semibold tracking-wider uppercase text-[#475467]">
          Trạng thái
        </span>
      </div>

      {/* Metric 1: Bong bóng bật */}
      <div className="p-3 rounded-xl bg-[#F8FAFC] border border-[#EAECF0] flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-[11.5px] font-medium text-[#64748B]">
            Bong bóng bật
          </span>
          <span className="text-[20px] font-bold text-[#1E293B] leading-tight">
            {totalConnected > 0
              ? `${visibleBubbleCount}/${totalConnected}`
              : "0"}
          </span>
        </div>
        <div
          className={`w-8 h-8 rounded-lg flex items-center justify-center ${
            visibleBubbleCount > 0
              ? "bg-[#0084FF]/10 text-[#0084FF]"
              : "bg-slate-200/60 text-slate-400"
          }`}
        >
          <span className="text-xs font-bold">
            {visibleBubbleCount > 0 ? "ON" : "OFF"}
          </span>
        </div>
      </div>

      {/* Metric 2: Tin chưa đọc */}
      <div className="p-3 rounded-xl bg-[#F8FAFC] border border-[#EAECF0] flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-[11.5px] font-medium text-[#64748B]">
            Tin chưa đọc
          </span>
          <span className="text-[20px] font-bold text-[#1E293B] leading-tight">
            {unreadCount}
          </span>
        </div>
        <div
          className={`px-2 py-1 rounded-md text-[11px] font-bold ${
            unreadCount > 0
              ? "bg-rose-50 text-rose-600 border border-rose-200/60"
              : "bg-slate-100 text-slate-400"
          }`}
        >
          {unreadCount > 0 ? `${unreadCount} mới` : "0"}
        </div>
      </div>
    </div>
  );
};
