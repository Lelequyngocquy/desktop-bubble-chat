import React from "react";
import { PanelsTopLeft, Info } from "lucide-react";
import { CommonLargeCard } from "../../common/index";

export const MessengerExplainerCard: React.FC = () => {
  return (
    <CommonLargeCard
      icon={<PanelsTopLeft className="w-4 h-4 text-[#0866FF]" />}
      title="Cơ chế Multi-Account"
      className="w-full xl:w-[380px] shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        <div className="space-y-3 text-[12px] text-[#475467] leading-relaxed">
          <div className="p-3 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/80 flex flex-col gap-1">
            <span className="font-semibold text-[#202733]">
              1. Độc lập hoàn toàn
            </span>
            <span>
              Mỗi tài khoản bạn thêm vào chạy trong một profile WebView2 biệt
              lập. Không chia sẻ cookie, không làm ảnh hưởng phiên làm việc của
              nhau.
            </span>
          </div>

          <div className="p-3 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/80 flex flex-col gap-1">
            <span className="font-semibold text-[#202733]">
              2. Đa màu sắc nhận diện
            </span>
            <span>
              Hệ thống tự động gán màu sắc riêng (M1 Xanh dương, M2 Tím, M3 Xanh
              ngọc, M4 Cam, M5 Đỏ san hô) để bạn không bao giờ nhầm lẫn.
            </span>
          </div>

          <div className="p-3 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/80 flex flex-col gap-1">
            <span className="font-semibold text-[#202733]">
              3. Chuyển đổi nhanh 1-Click
            </span>
            <span>
              Nhấn trực tiếp vào bong bóng chat hoặc nút Mở chat để gọi ngay
              khung hội thoại của tài khoản tương ứng.
            </span>
          </div>
        </div>
      </div>

      <div className="p-3 bg-[#EEF2F9]/50 rounded-[8px] flex items-start gap-2 text-[11px] text-[#6B7483]">
        <Info className="w-3.5 h-3.5 shrink-0 text-[#0866FF] mt-0.5" />
        <span>
          Bạn có thể thay đổi kích thước bong bóng hoặc cài đặt phím tắt toàn
          cục trong tab Cài đặt.
        </span>
      </div>
    </CommonLargeCard>
  );
};
