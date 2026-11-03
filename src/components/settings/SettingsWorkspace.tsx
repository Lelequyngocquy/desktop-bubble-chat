import React, { useState } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUpdateStore } from "../../stores/updateStore";
import { useQuickSettings } from "../../hooks/useQuickSettings";
import { messengerService } from "../../platforms/messenger";
import { TabHeader } from "../common";
import {
  SystemStartupSection,
  BubbleCustomizationSection,
  GlobalShortcutSection,
  DataCacheSection,
  AppUpdateSection,
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

  const isGlobalShortcutEnabled = useSettingsStore(
    (s) => s.isGlobalShortcutEnabled,
  );
  const toggleGlobalShortcut = useSettingsStore((s) => s.toggleGlobalShortcut);
  const currentVersion = useUpdateStore((s) => s.currentVersion);
  const hasUpdate = useUpdateStore(
    (s) =>
      s.status === "available" ||
      s.status === "downloading" ||
      s.status === "downloaded",
  );

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

  const scrollToUpdateSection = () => {
    document
      .getElementById("section-update")
      ?.scrollIntoView({ behavior: "smooth" });
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="TÙY CHỈNH HỆ THỐNG"
        title="Cài đặt"
        description="Tùy biến cấu hình hệ thống, kích thước bong bóng nổi, phím tắt, cập nhật và bộ nhớ đệm."
        rightSlot={
          <button
            onClick={scrollToUpdateSection}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-background-surface hover:bg-white border border-border rounded-pill text-small shadow-xs transition-colors cursor-pointer"
            title="Nhấn để cuộn đến khu vực Cập nhật ứng dụng"
          >
            <span
              className={`w-2 h-2 rounded-pill ${
                hasUpdate ? "bg-primary animate-pulse" : "bg-status-success"
              }`}
            />
            <span className="text-content-primary font-medium">
              Phiên bản v{currentVersion}
            </span>
          </button>
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

        <div id="section-update">
          <AppUpdateSection />
        </div>
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
