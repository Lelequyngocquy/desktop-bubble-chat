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
import { useNavigationStore } from "../../../stores/navigationStore";
import { useQuickSettings } from "../../../hooks/useQuickSettings";
import { CommonWidget } from "../../common";
import { QUICK_SETTINGS_CONFIG, QuickSettingItem } from "../../../config";
import { QuickSettingRow } from "./QuickSettingRow";

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
    MessageSquare: (
      <MessageSquare className="w-[18px] h-[18px] text-[#6B7483]" />
    ),
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

  // Tính số lượng thiết lập đang được bật
  const activeCount = settings.filter((s) => s.value).length;

  return (
    <CommonWidget
      className="w-full lg:w-[432px] shrink-0 h-[380px] lg:h-full"
      title="Thiết lập nhanh"
      headerAction={
        <SlidersHorizontal className="w-[18px] h-[18px] text-[#6B7483]" />
      }
      banner={
        warningMessage ? (
          <div
            onClick={() => setActiveTab("connections")}
            className="p-2 bg-[#FFF3E8] border border-[#FCD34D] rounded-[6px] flex items-center gap-2 text-[#D97706] text-[11px] cursor-pointer hover:bg-[#FEEBC8] transition-colors"
          >
            <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
            <span className="flex-1 font-medium">{warningMessage}</span>
            <span className="text-[10px] underline font-semibold">
              Kết nối ngay
            </span>
          </div>
        ) : undefined
      }
      bodyClassName="py-1 flex flex-col justify-between"
      footerLeft={
        <span className="text-[11px] text-[#6B7483]">
          {activeCount}/{settings.length} tính năng đang bật
        </span>
      }
      footerRight={
        <button
          type="button"
          onClick={() => setActiveTab("settings")}
          className="flex items-center gap-1.5 text-[12px] font-medium text-[#0759DB] hover:text-[#0866FF] transition-colors cursor-pointer"
        >
          <span>Tất cả cài đặt</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      }
    >
      {settings.map((item) => (
        <QuickSettingRow
          key={item.id}
          label={item.label}
          description={item.description}
          icon={item.icon}
          value={item.value}
          disabled={item.disabled}
          onChange={item.onChange}
        />
      ))}
    </CommonWidget>
  );
};
