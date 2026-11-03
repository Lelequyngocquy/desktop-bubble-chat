import React from "react";
import { PanelsTopLeft, Info } from "lucide-react";
import { CommonLargeCard } from "../../common/index";

export const MessengerExplainerCard: React.FC = () => {
  return (
    <CommonLargeCard
      icon={<PanelsTopLeft className="w-4 h-4 text-primary" />}
      title="Cơ chế Multi-Account"
      className="w-full shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        <div className="space-y-3 text-small text-content-secondary leading-relaxed">
          <div className="p-3 bg-background-subtle rounded-panel border border-border/80 flex flex-col gap-1">
            <span className="font-semibold text-content-primary">
              1. Độc lập hoàn toàn
            </span>
            <span>
              Mỗi tài khoản bạn thêm vào chạy trong một profile WebView2 biệt
              lập. Không chia sẻ cookie, không làm ảnh hưởng phiên làm việc của
              nhau.
            </span>
          </div>

          <div className="p-3 bg-background-subtle rounded-panel border border-border/80 flex flex-col gap-1">
            <span className="font-semibold text-content-primary">
              2. Đa màu sắc nhận diện
            </span>
            <span>
              Hệ thống tự động gán màu sắc riêng để bạn không bao giờ nhầm lẫn.
            </span>
          </div>

          <div className="p-3 bg-background-subtle rounded-panel border border-border/80 flex flex-col gap-1">
            <span className="font-semibold text-content-primary">
              3. Chuyển đổi nhanh 1-Click
            </span>
            <span>
              Trỏ con trỏ chuột trực tiếp vào bong bóng chat hoặc nút Mở chat
              (trang Kết nối) để gọi ngay khung hội thoại của tài khoản tương
              ứng.
            </span>
          </div>
        </div>
      </div>

      <div className="p-3 bg-background-app/50 rounded-panel flex items-start gap-2 text-caption text-content-secondary">
        <Info className="w-3.5 h-3.5 shrink-0 text-primary mt-0.5" />
        <span>
          Bạn có thể thay đổi kích thước bong bóng hoặc cài đặt phím tắt toàn
          cục trong tab Cài đặt.
        </span>
      </div>
    </CommonLargeCard>
  );
};

export default MessengerExplainerCard;
