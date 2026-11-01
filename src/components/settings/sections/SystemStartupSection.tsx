import React from "react";
import { Power } from "lucide-react";
import { SwitchButton, CommonLargeCard } from "../../common";

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
    <CommonLargeCard
      icon={<Power className="w-4 h-4 text-[#0866FF]" />}
      title="Hệ thống & Khởi động"
    >
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
    </CommonLargeCard>
  );
};

export default SystemStartupSection;
