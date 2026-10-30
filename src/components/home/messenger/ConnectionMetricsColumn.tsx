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
    <div className="w-full lg:w-[240px] shrink-0 border-t lg:border-t-0 lg:border-l border-[#E2E7EF] pt-4 lg:pt-0 lg:pl-6 flex flex-col justify-center gap-3">
      <div className="flex items-center gap-1.5 text-[#6B7483]">
        <Activity className="w-[14px] h-[14px]" />
        <span className="text-[11px] font-medium tracking-wider uppercase">
          Trạng thái hiện tại
        </span>
      </div>

      <div className="flex flex-col gap-4">
        <div className="flex flex-col">
          <span className="text-[26px] font-bold text-[#202733] leading-none">
            {totalConnected > 0 ? `${visibleBubbleCount}/${totalConnected}` : 0}
          </span>
          <span className="text-[12px] text-[#6B7483] mt-1">
            Bong bóng đang bật
          </span>
        </div>

        <div className="flex flex-col">
          <span className="text-[26px] font-bold text-[#202733] leading-none">
            {unreadCount}
          </span>
          <span className="text-[12px] text-[#6B7483] mt-1">Tin chưa đọc</span>
        </div>
      </div>
    </div>
  );
};
