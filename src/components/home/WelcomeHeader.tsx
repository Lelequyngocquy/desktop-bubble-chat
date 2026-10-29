import React from "react";
import { MonitorCheck } from "lucide-react";

export const WelcomeHeader: React.FC = () => {
  return (
    <div className="flex items-center justify-between gap-4 select-none">
      {/* Welcome text block (Figma Node #3:1200) */}
      <div className="flex flex-col gap-[5px]">
        <span className="text-[12px] font-medium text-[#6B7483] uppercase tracking-wider">
          KHÔNG GIAN CỦA BẠN
        </span>
        <h1 className="text-[30px] font-semibold text-[#202733] leading-tight m-0">
          Chào Minh Anh,
        </h1>
        <p className="text-[14px] text-[#6B7483] m-0 leading-normal">
          Mọi kết nối ở đây. Mọi cuộc trò chuyện luôn trong tầm tay.
        </p>
      </div>

      {/* Desktop state badge (Figma Node #3:1204) */}
      <div className="flex items-center gap-2 px-[13px] py-2 bg-white border border-[#E2E7EF] rounded-full shadow-xs shrink-0">
        <MonitorCheck className="w-4 h-4 text-[#6B7483]" />
        <span className="text-[12px] text-[#6B7483] font-normal">
          Sẵn sàng trên desktop
        </span>
      </div>
    </div>
  );
};
