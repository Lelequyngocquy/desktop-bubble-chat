import React from "react";
import { Cpu, Sparkles, XCircle, MousePointerClick, Menu, Info } from "lucide-react";
import { CommonLargeCard } from "../../common/index";

export const ResourceOptimizationCard: React.FC = () => {
  return (
    <CommonLargeCard
      icon={<Cpu className="w-4 h-4 text-primary" />}
      title="Gợi ý tối ưu tài nguyên: Chạy ẩn Dashboard"
      subTitle={
        <span className="px-2.5 py-0.5 bg-status-successBg text-status-success border border-status-successBorder rounded-pill text-caption font-semibold inline-flex items-center gap-1 shadow-2xs">
          <Sparkles className="w-3 h-3 text-status-success" />
          Khuyên dùng
        </span>
      }
      className="w-full shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        {/* Banner tóm tắt lợi ích tối ưu */}
        <div className="p-3.5 bg-status-successBg rounded-card-sm border border-status-successBorder flex items-start gap-3">
          <div className="w-8 h-8 rounded-panel bg-status-successBg text-status-success flex items-center justify-center shrink-0 mt-0.5">
            <Cpu className="w-4 h-4" />
          </div>
          <div className="flex flex-col gap-1 text-small text-status-success leading-relaxed">
            <span className="font-semibold text-status-success">
              Giải phóng RAM & CPU khi không cần xem cài đặt
            </span>
            <span>
              Sau khi đã bật các bong bóng chat, bạn nên <strong>bấm nút [X] để đóng Dashboard</strong>.
              Giao diện trang quản lý sẽ được giải phóng hoàn toàn khỏi bộ nhớ, trong khi các Bong bóng Win32 siêu nhẹ
              vẫn duy trì kết nối nhận tin nhắn và thông báo liên tục 24/7.
            </span>
          </div>
        </div>

        {/* 3 Khối hướng dẫn chi tiết cách làm việc với Khay hệ thống (Tray Icon) */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-small text-content-secondary leading-relaxed">
          {/* Bước 1 */}
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
                <XCircle className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                1. Đóng ẩn Dashboard
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              Nhấn nút <strong>[X]</strong> ở góc trên bên phải cửa sổ. Ứng dụng <strong>không bị thoát</strong> mà tự động thu nhỏ về chạy ngầm dưới Khay hệ thống (System Tray).
            </span>
          </div>

          {/* Bước 2 */}
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
                <MousePointerClick className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                2. Mở lại Dashboard
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              Khi cần thay đổi cấu hình, <strong>nhấp đúp chuột trái (Double-click)</strong> vào biểu tượng ứng dụng ở góc phải thanh Taskbar (gần đồng hồ Windows) để mở lại giao diện.
            </span>
          </div>

          {/* Bước 3 */}
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
                <Menu className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                3. Menu Khay hệ thống
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              <strong>Nhấp chuột phải</strong> vào biểu tượng khay để bật/tắt nhanh từng bong bóng, ẩn tất cả bong bóng khi cần tập trung, hoặc chọn <em>&quot;Thoát ứng dụng&quot;</em> để dừng hoàn toàn.
            </span>
          </div>
        </div>
      </div>

      {/* Ghi chú chân trang */}
      <div className="p-3 bg-background-app/60 rounded-panel flex items-start gap-2 text-caption text-content-secondary">
        <Info className="w-3.5 h-3.5 shrink-0 text-primary mt-0.5" />
        <span>
          <strong>Mẹo tìm biểu tượng:</strong> Nếu không thấy biểu tượng ứng dụng trên Taskbar, hãy bấm vào mũi tên <strong>^ (Show hidden icons)</strong> ở góc phải khay hệ thống Windows.
        </span>
      </div>
    </CommonLargeCard>
  );
};

export default ResourceOptimizationCard;
