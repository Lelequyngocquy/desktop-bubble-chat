import React, { useEffect } from "react";
import {
  SlidersHorizontal,
  MessageSquare,
  Pin,
  Bell,
  Power,
  ArrowRight,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/appStore";
import { SwitchButton } from "../common";

export const QuickSettings: React.FC = () => {
  const {
    isBubbleEnabled,
    toggleBubble,
    isAlwaysOnTop,
    toggleAlwaysOnTop,
    isNotificationsEnabled,
    toggleNotifications,
    isAutoStart,
    setAutoStart,
  } = useAppStore();

  useEffect(() => {
    // Truy vấn trạng thái khởi động cùng Windows thực tế từ Registry
    invoke<boolean>("is_autostart_enabled")
      .then((enabled) => {
        setAutoStart(enabled);
      })
      .catch(() => {
        // Môi trường dev hoặc browser preview
      });
  }, [setAutoStart]);

  const handleToggleBubble = async () => {
    const nextVal = !isBubbleEnabled;
    toggleBubble();
    try {
      await invoke("toggle_bubble", { visible: nextVal });
    } catch {
      // In standard browser preview
    }
  };

  const handleToggleAutoStart = async () => {
    const nextVal = !isAutoStart;
    setAutoStart(nextVal);
    try {
      await invoke("toggle_autostart", { enabled: nextVal });
    } catch {
      // In standard browser preview
    }
  };

  const settings = [
    {
      id: "bubble",
      label: "Bong bóng nổi",
      description: "Hiển thị chat heads trên desktop",
      icon: <MessageSquare className="w-[18px] h-[18px] text-[#6B7483]" />,
      value: isBubbleEnabled,
      onChange: handleToggleBubble,
    },
    {
      id: "always-on-top",
      label: "Luôn nổi trên cùng",
      description: "Giữ phía trên các cửa sổ khác",
      icon: <Pin className="w-[18px] h-[18px] text-[#6B7483]" />,
      value: isAlwaysOnTop,
      onChange: toggleAlwaysOnTop,
    },
    {
      id: "notifications",
      label: "Thông báo",
      description: "Nhận thông báo khi có tin nhắn mới",
      icon: <Bell className="w-[18px] h-[18px] text-[#6B7483]" />,
      value: isNotificationsEnabled,
      onChange: toggleNotifications,
    },
    {
      id: "autostart",
      label: "Khởi động cùng Windows",
      description: "Sẵn sàng ngay khi bạn đăng nhập",
      icon: <Power className="w-[18px] h-[18px] text-[#6B7483]" />,
      value: isAutoStart,
      onChange: handleToggleAutoStart,
    },
  ];

  return (
    <div className="w-full lg:w-[432px] bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-[20px_22px_12px] flex flex-col justify-between shrink-0 min-h-[352px] select-none">
      {/* Heading (Figma Node #3:1300) */}
      <div className="flex items-center justify-between pb-2 border-b border-[#E2E7EF]/50">
        <h2 className="text-[17px] font-semibold text-[#202733] m-0">
          Thiết lập nhanh
        </h2>
        <SlidersHorizontal className="w-[18px] h-[18px] text-[#6B7483]" />
      </div>

      {/* Settings List (Figma Node #3:1303) */}
      <div className="flex flex-col py-1">
        {settings.map((item) => (
          <div
            key={item.id}
            className="h-[61px] flex items-center justify-between gap-3 group"
          >
            <div className="flex items-center gap-3">
              <div className="w-7 h-7 flex items-center justify-center shrink-0">
                {item.icon}
              </div>
              <div className="flex flex-col gap-0.5">
                <span className="text-[13px] font-medium text-[#202733] leading-tight">
                  {item.label}
                </span>
                <span className="text-[11px] text-[#6B7483] leading-tight">
                  {item.description}
                </span>
              </div>
            </div>

            {/* Toggle Switch Component (Figma Node #3:1309 / #3:1330) */}
            <SwitchButton
              checked={item.value}
              onChange={item.onChange}
              ariaLabel={item.label}
            />
          </div>
        ))}
      </div>

      {/* All Settings link (Figma Node #3:1332) */}
      <div className="h-[42px] border-t border-[#E2E7EF] flex items-center gap-1.5 text-[#0759DB] hover:text-[#0866FF] transition-colors cursor-pointer pt-1">
        <span className="text-[12px] font-medium">Tất cả cài đặt</span>
        <ArrowRight className="w-3.5 h-3.5" />
      </div>
    </div>
  );
};
