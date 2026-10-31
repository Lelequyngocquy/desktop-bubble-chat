import React from "react";
import { Power } from "lucide-react";
import { SwitchButton } from "../../common";

interface SystemStartupSectionProps {
  isAutoStart: boolean;
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  onToggleAutoStart: () => void | Promise<void>;
  onToggleAlwaysOnTop: () => void | Promise<void>;
  onToggleNotifications: () => void | Promise<void>;
}

export const SystemStartupSection: React.FC<SystemStartupSectionProps> = ({
  isAutoStart,
  isAlwaysOnTop,
  isNotificationsEnabled,
  onToggleAutoStart,
  onToggleAlwaysOnTop,
  onToggleNotifications,
}) => {
  return (
    <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
      <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
        <Power className="w-4 h-4 text-[#0866FF]" />
        <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
          Hệ thống & Khởi động
        </h2>
      </div>

      <div className="flex flex-col divide-y divide-[#E2E7EF]">
        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-[13px] font-medium text-[#1E293B]">
              Khởi động cùng Windows
            </span>
            <span className="text-[11px] text-[#64748B]">
              Tự động bật DesktopBubbleChat khi bạn đăng nhập vào máy tính
            </span>
          </div>
          <SwitchButton
            checked={isAutoStart}
            onChange={onToggleAutoStart}
          />
        </div>

        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-[13px] font-medium text-[#1E293B]">
              Bong bóng nổi trên cùng
            </span>
            <span className="text-[11px] text-[#64748B]">
              Khi tắt, các ứng dụng khác có thể che phủ bong bóng nhưng vẫn
              mở khung trò chuyện nếu di chuột đúng vị trí
            </span>
          </div>
          <SwitchButton
            checked={isAlwaysOnTop}
            onChange={onToggleAlwaysOnTop}
          />
        </div>

        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-[13px] font-medium text-[#1E293B]">
              Thông báo desktop & Âm thanh
            </span>
            <span className="text-[11px] text-[#64748B]">
              Phát âm thanh bong bóng nhẹ và hiển thị thông báo toast khi có
              tin nhắn mới
            </span>
          </div>
          <SwitchButton
            checked={isNotificationsEnabled}
            onChange={onToggleNotifications}
          />
        </div>
      </div>
    </div>
  );
};

export default SystemStartupSection;
