import React, { useState } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { useQuickSettings } from "../../hooks/useQuickSettings";
import { messengerService } from "../../platforms/messenger";
import { TabHeader } from "../common";
import {
  SystemStartupSection,
  BubbleCustomizationSection,
  GlobalShortcutSection,
  DataCacheSection,
} from "./sections";
import { ClearCacheDialog } from "./dialogs";

export const SettingsWorkspace: React.FC = () => {
  const {
    isAlwaysOnTop,
    handleToggleAlwaysOnTop,
    isNotificationsEnabled,
    handleToggleNotifications,
    isAutoStart,
    handleToggleAutoStart,
    bubbleSize,
    handleSetBubbleSize,
  } = useQuickSettings();

  const isGlobalShortcutEnabled = useSettingsStore((s) => s.isGlobalShortcutEnabled);
  const toggleGlobalShortcut = useSettingsStore((s) => s.toggleGlobalShortcut);

  const [clearStatus, setClearStatus] = useState<string | null>(null);
  const [isConfirmClearCacheOpen, setIsConfirmClearCacheOpen] = useState(false);

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
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="TÙY CHỈNH HỆ THỐNG"
        title="Cài đặt"
        description="Tùy biến cấu hình hệ thống, kích thước bong bóng nổi, phím tắt và bộ nhớ đệm."
        rightSlot={
          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
            <span className="w-2 h-2 rounded-pill bg-status-success" />
            <span className="text-content-primary font-medium">Phiên bản v0.1.0</span>
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
          onSelectSize={handleSetBubbleSize}
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
