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
      icon={<Power className="w-4 h-4 text-primary" />}
      title="Hệ thống & Khởi động"
    >
      <div className="flex flex-col divide-y divide-border">
        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary">
              Khởi động cùng Windows
            </span>
            <span className="text-caption text-content-secondary">
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
            <span className="text-body font-medium text-content-primary">
              Bong bóng nổi trên cùng
            </span>
            <span className="text-caption text-content-secondary">
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
            <span className="text-body font-medium text-content-primary">
              Thông báo desktop & Âm thanh
            </span>
            <span className="text-caption text-content-secondary">
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
