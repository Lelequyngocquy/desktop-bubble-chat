import React from "react";
import { PanelsTopLeft, Info } from "lucide-react";

export const ConnectionExplainerCard: React.FC = () => {
  return (
    <div className="w-full xl:w-[344px] shrink-0 bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-[22px] flex flex-col gap-4">
      <div className="flex items-center gap-2.5 pb-2 border-b border-[#E2E7EF]/50">
        <PanelsTopLeft className="w-[18px] h-[18px] text-[#6B7483]" />
        <h3 className="text-[16px] font-semibold text-[#202733] m-0">
          Kết nối hoạt động thế nào?
        </h3>
      </div>

      <div className="flex flex-col gap-1">
        <span className="text-[13px] font-semibold text-[#202733]">
          Smart WebView Wrapper
        </span>
        <p className="text-[12px] text-[#6B7483] leading-relaxed m-0">
          Hãy đăng nhập trước để đảm bảo có thể mở bong bóng chat!
        </p>
      </div>

      <div className="flex flex-col gap-1">
        <span className="text-[13px] font-semibold text-[#202733]">
          Phiên đăng nhập trên thiết bị
        </span>
        <p className="text-[12px] text-[#6B7483] leading-relaxed m-0">
          Phiên được lưu để bạn tiếp tục sử dụng trên máy này. Messenger có thể
          yêu cầu đăng nhập lại khi phiên hết hạn.
        </p>
      </div>

      <div className="p-3 bg-[#EAF2FF] rounded-[8px] flex items-start gap-2 text-[#0759DB] mt-auto">
        <Info className="w-4 h-4 shrink-0 mt-0.5" />
        <p className="text-[11px] leading-snug m-0">
          Ngắt kết nối sẽ dừng đồng bộ Messenger trong DesktopBubbleChat.
        </p>
      </div>
    </div>
  );
};
