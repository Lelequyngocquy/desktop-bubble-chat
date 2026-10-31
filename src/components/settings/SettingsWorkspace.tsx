import React, { useState } from "react";
import {
  useSettingsStore,
  BubbleSize,
  BUBBLE_SIZE_PIXELS,
} from "../../stores/settingsStore";
import { systemService } from "../../services/systemService";
import { messengerService } from "../../services/messenger";
import { notificationService } from "../../services/notificationService";
import { TabHeader } from "../common";
import {
  SystemStartupSection,
  BubbleCustomizationSection,
  GlobalShortcutSection,
  DataCacheSection,
} from "./sections";
import { ClearCacheDialog } from "./dialogs";

export const SettingsWorkspace: React.FC = () => {
  const isAlwaysOnTop = useSettingsStore((s) => s.isAlwaysOnTop);
  const toggleAlwaysOnTop = useSettingsStore((s) => s.toggleAlwaysOnTop);
  const isNotificationsEnabled = useSettingsStore((s) => s.isNotificationsEnabled);
  const toggleNotifications = useSettingsStore((s) => s.toggleNotifications);
  const isAutoStart = useSettingsStore((s) => s.isAutoStart);
  const toggleAutoStart = useSettingsStore((s) => s.toggleAutoStart);
  const isGlobalShortcutEnabled = useSettingsStore((s) => s.isGlobalShortcutEnabled);
  const toggleGlobalShortcut = useSettingsStore((s) => s.toggleGlobalShortcut);
  const bubbleSize = useSettingsStore((s) => s.bubbleSize);
  const setBubbleSize = useSettingsStore((s) => s.setBubbleSize);

  const [clearStatus, setClearStatus] = useState<string | null>(null);
  const [isConfirmClearCacheOpen, setIsConfirmClearCacheOpen] = useState(false);

  const handleToggleAutoStart = async () => {
    const nextVal = !isAutoStart;
    toggleAutoStart();
    await systemService.toggleAutostart(nextVal);
  };

  const handleToggleAlwaysOnTop = async () => {
    const nextVal = !isAlwaysOnTop;
    toggleAlwaysOnTop();
    await systemService.setAlwaysOnTop(nextVal);
  };

  const handleToggleNotifications = async () => {
    const nextVal = !isNotificationsEnabled;
    toggleNotifications();
    if (nextVal) {
      await notificationService.requestPermission();
    }
  };

  const handleSelectBubbleSize = async (size: BubbleSize) => {
    setBubbleSize(size);
    await messengerService.setBubbleConfig(BUBBLE_SIZE_PIXELS[size]);
  };

  const handleClearCache = async () => {
    try {
      setClearStatus("clearing");
      await messengerService.clearStorage(1);
      setClearStatus("success");
      setTimeout(() => setClearStatus(null), 3000);
    } catch {
      setClearStatus("error");
      setTimeout(() => setClearStatus(null), 3000);
    }
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="TÙY CHỈNH HỆ THỐNG"
        title="Cài đặt"
        description="Tùy biến cấu hình hệ thống, kích thước bong bóng nổi, phím tắt và bộ nhớ đệm."
        rightSlot={
          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-full text-[12px] shadow-xs">
            <span className="w-2 h-2 rounded-full bg-[#10B981]" />
            <span className="text-[#202733] font-medium">Phiên bản v0.1.0</span>
          </div>
        }
      />

      {/* 2. Nội dung cuộn mượt mà các Section chức năng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <SystemStartupSection
          isAutoStart={isAutoStart}
          isAlwaysOnTop={isAlwaysOnTop}
          isNotificationsEnabled={isNotificationsEnabled}
          onToggleAutoStart={handleToggleAutoStart}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          onToggleNotifications={handleToggleNotifications}
        />

        <BubbleCustomizationSection
          bubbleSize={bubbleSize}
          onSelectSize={handleSelectBubbleSize}
        />

        <GlobalShortcutSection
          isGlobalShortcutEnabled={isGlobalShortcutEnabled}
          onToggleGlobalShortcut={toggleGlobalShortcut}
        />

        <DataCacheSection
          clearStatus={clearStatus}
          onOpenClearDialog={() => setIsConfirmClearCacheOpen(true)}
        />
      </main>

      {/* 3. Dialog xác nhận xóa cache */}
      <ClearCacheDialog
        isOpen={isConfirmClearCacheOpen}
        isClearing={clearStatus === "clearing"}
        onClose={() => setIsConfirmClearCacheOpen(false)}
        onConfirm={async () => {
          setIsConfirmClearCacheOpen(false);
          await handleClearCache();
        }}
      />
    </div>
  );
};

export default SettingsWorkspace;
