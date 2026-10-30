import React from "react";
import {
  SlidersHorizontal,
  MessageSquare,
  Pin,
  Bell,
  Power,
  ArrowRight,
  AlertTriangle,
} from "lucide-react";
import { useNavigationStore } from "../../stores/navigationStore";
import { useQuickSettings } from "../../hooks/useQuickSettings";
import { SwitchButton, CommonWidget } from "../common";
import { QUICK_SETTINGS_CONFIG, QuickSettingItem } from "../../config";

export const QuickSettings: React.FC = () => {
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const {
    isBubbleEnabled,
    bubbleDescription,
    isAlwaysOnTop,
    isNotificationsEnabled,
    isAutoStart,
    isConnected,
    warningMessage,
    handleToggleBubble,
    handleToggleAlwaysOnTop,
    handleToggleNotifications,
    handleToggleAutoStart,
  } = useQuickSettings();

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
      disabled: !isConnected,
    },
    "always-on-top": {
      value: isAlwaysOnTop,
      onChange: handleToggleAlwaysOnTop,
    },
    notifications: {
      value: isNotificationsEnabled,
      onChange: handleToggleNotifications,
    },
    autostart: {
      value: isAutoStart,
      onChange: handleToggleAutoStart,
    },
  };

  const settings = QUICK_SETTINGS_CONFIG.map((item) => {
    const action = actionMap[item.id];
    const isAvailable = !item.requiresConnection || isConnected;
    const description =
      item.id === "bubble"
        ? bubbleDescription
        : isAvailable
          ? item.descriptionActive
          : item.descriptionInactive;

    return {
      id: item.id,
      label: item.label,
      description,
      icon: iconMap[item.iconName],
      value: action?.value ?? false,
      onChange: action?.onChange ?? (() => {}),
      disabled: action?.disabled ?? false,
    };
  });

  return (
    <CommonWidget
      className="w-full lg:w-[432px] shrink-0"
      title="Thiết lập nhanh"
      headerAction={<SlidersHorizontal className="w-[18px] h-[18px] text-[#6B7483]" />}
      banner={
        warningMessage ? (
          <div
            onClick={() => setActiveTab("connections")}
            className="p-2 bg-[#FFF3E8] border border-[#FCD34D] rounded-[6px] flex items-center gap-2 text-[#D97706] text-[11px] cursor-pointer hover:bg-[#FEEBC8] transition-colors"
          >
            <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
            <span className="flex-1 font-medium">{warningMessage}</span>
            <span className="text-[10px] underline font-semibold">Kết nối ngay</span>
          </div>
        ) : undefined
      }
      bodyClassName="py-1"
      footerRight={
        <button
          type="button"
          onClick={() => setActiveTab("settings")}
          className="flex items-center gap-1.5 text-[12px] font-medium text-[#0759DB] hover:text-[#0866FF] transition-colors"
        >
          <span>Tất cả cài đặt</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      }
    >
      {settings.map((item) => (
        <div key={item.id} className="flex flex-col">
          <div
            className={`h-[56px] flex items-center justify-between gap-3 group ${
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
        </div>
      ))}
    </CommonWidget>
  );
};