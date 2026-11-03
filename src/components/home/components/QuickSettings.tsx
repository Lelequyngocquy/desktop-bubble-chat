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
    notificationsDescription,
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
      <MessageSquare className="w-[18px] h-[18px] text-content-secondary" />
    ),
    Pin: <Pin className="w-[18px] h-[18px] text-content-secondary" />,
    Bell: <Bell className="w-[18px] h-[18px] text-content-secondary" />,
    Power: <Power className="w-[18px] h-[18px] text-content-secondary" />,
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
        : item.id === "notifications"
          ? notificationsDescription
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
      className="w-full lg:w-[432px] shrink-0 h-side-col lg:h-full"
      title="Thiết lập nhanh"
      headerAction={
        <SlidersHorizontal className="w-[18px] h-[18px] text-content-secondary" />
      }
      banner={
        warningMessage ? (
          <div
            onClick={() => setActiveTab("connections")}
            className="p-2 bg-status-warningBg border border-status-warningBorder rounded-control flex items-center gap-2 text-status-warning text-caption cursor-pointer hover:bg-status-warningBg/80 transition-colors"
          >
            <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
            <span className="flex-1 font-medium">{warningMessage}</span>
            <span className="text-badge underline font-semibold">
              Kết nối ngay
            </span>
          </div>
        ) : undefined
      }
      bodyClassName="py-1 flex flex-col justify-between"
      footerLeft={
        <span className="text-caption text-content-secondary">
          {activeCount}/{settings.length} tính năng đang bật
        </span>
      }
      footerRight={
        <button
          type="button"
          onClick={() => setActiveTab("settings")}
          className="flex items-center gap-1.5 text-small font-medium text-primary-foreground hover:text-primary transition-colors cursor-pointer"
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

export default QuickSettings;
