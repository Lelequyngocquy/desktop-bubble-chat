import React, { useState, useEffect } from "react";
import {
  SlidersHorizontal,
  MessageSquare,
  Pin,
  Bell,
  Power,
  ArrowRight,
  AlertTriangle,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/appStore";
import { SwitchButton } from "../common";
import { MOCK_QUICK_SETTINGS_SCHEMA, QuickSettingItem } from "../../mocks";

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
    isMessengerConnected,
    setActiveTab,
  } = useAppStore();

  const [warningMessage, setWarningMessage] = useState<string | null>(null);

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
    if (!isMessengerConnected && !isBubbleEnabled) {
      setWarningMessage("Vui lòng kết nối Messenger trước khi bật bong bóng chat!");
      setTimeout(() => setWarningMessage(null), 3500);
      return;
    }
    const success = toggleBubble();
    if (!success) return;

    const nextVal = !isBubbleEnabled;
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

  const iconMap: Record<QuickSettingItem["iconName"], React.ReactNode> = {
    MessageSquare: <MessageSquare className="w-[18px] h-[18px] text-[#6B7483]" />,
    Pin: <Pin className="w-[18px] h-[18px] text-[#6B7483]" />,
    Bell: <Bell className="w-[18px] h-[18px] text-[#6B7483]" />,
    Power: <Power className="w-[18px] h-[18px] text-[#6B7483]" />,
  };

  const actionMap: Record<
    string,
    { value: boolean; onChange: () => void | Promise<void>; disabled?: boolean }
  > = {
    bubble: {
      value: isBubbleEnabled,
      onChange: handleToggleBubble,
      disabled: !isMessengerConnected,
    },
    "always-on-top": {
      value: isAlwaysOnTop,
      onChange: toggleAlwaysOnTop,
    },
    notifications: {
      value: isNotificationsEnabled,
      onChange: toggleNotifications,
    },
    autostart: {
      value: isAutoStart,
      onChange: handleToggleAutoStart,
    },
  };

  const settings = MOCK_QUICK_SETTINGS_SCHEMA.map((item) => {
    const action = actionMap[item.id];
    const isAvailable = !item.requiresConnection || isMessengerConnected;
    return {
      id: item.id,
      label: item.label,
      description: isAvailable ? item.descriptionActive : item.descriptionInactive,
      icon: iconMap[item.iconName],
      value: action?.value ?? false,
      onChange: action?.onChange ?? (() => {}),
      disabled: action?.disabled ?? false,
    };
  });

  return (
    <div className="w-full lg:w-[432px] bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-[20px_22px_12px] flex flex-col justify-between shrink-0 min-h-[352px] select-none">
      {/* Heading (Figma Node #3:1300) */}
      <div className="flex items-center justify-between pb-2 border-b border-[#E2E7EF]/50">
        <h2 className="text-[17px] font-semibold text-[#202733] m-0">
          Thiết lập nhanh
        </h2>
        <SlidersHorizontal className="w-[18px] h-[18px] text-[#6B7483]" />
      </div>

      {/* Warning message if trying to turn on bubble without Messenger */}
      {warningMessage && (
        <div
          onClick={() => setActiveTab("connections")}
          className="my-1.5 p-2 bg-[#FFF3E8] border border-[#FCD34D] rounded-[6px] flex items-center gap-2 text-[#D97706] text-[11px] cursor-pointer hover:bg-[#FEEBC8] transition-colors"
        >
          <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
          <span className="flex-1 font-medium">{warningMessage}</span>
          <span className="text-[10px] underline font-semibold">Kết nối ngay</span>
        </div>
      )}

      {/* Settings List (Figma Node #3:1303) */}
      <div className="flex flex-col py-1">
        {settings.map((item) => (
          <div
            key={item.id}
            className={`h-[61px] flex items-center justify-between gap-3 group ${
              item.disabled ? "opacity-60" : ""
            }`}
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

            <SwitchButton
              checked={item.value}
              onChange={item.onChange}
              disabled={item.disabled}
              aria-label={item.label}
            />
          </div>
        ))}
      </div>

      {/* Footer Settings link (Figma Node #3:1330) */}
      <div className="h-12 border-t border-[#E2E7EF]/50 flex items-center justify-end">
        <button
          type="button"
          onClick={() => setActiveTab("settings")}
          className="flex items-center gap-1.5 text-[12px] font-medium text-[#0759DB] hover:text-[#0866FF] transition-colors"
        >
          <span>Tất cả cài đặt</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
