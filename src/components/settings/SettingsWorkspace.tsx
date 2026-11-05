import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUpdateStore } from "../../stores/updateStore";
import { useQuickSettings } from "../../hooks/useQuickSettings";
import { messengerService } from "../../platforms/messenger";
import { TabHeader } from "../common";
import {
  LanguageSelectionSection,
  AppearanceSection,
  SystemStartupSection,
  BubbleCustomizationSection,
  GlobalShortcutSection,
  DataCacheSection,
  AppUpdateSection,
} from "./sections";
import { ClearCacheDialog } from "./dialogs";

export const SettingsWorkspace: React.FC = () => {
  const { t } = useTranslation(["settings", "common"]);
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
  const isAutoTrimEnabled = useSettingsStore((s) => s.isAutoTrimEnabled);
  const toggleAutoTrim = useSettingsStore((s) => s.toggleAutoTrim);
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
      {/* Settings workspace header */}
      <TabHeader
        eyebrow={t("settings:title").toUpperCase()}
        title={t("settings:title")}
        description={t("settings:subtitle")}
        rightSlot={
          <button
            onClick={scrollToUpdateSection}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-background-surface hover:bg-background-subtle border border-border rounded-pill text-small shadow-xs transition-colors cursor-pointer"
            title={t("settings:scroll_to_update")}
          >
            <span
              className={`w-2 h-2 rounded-pill ${
                hasUpdate ? "bg-primary animate-pulse" : "bg-status-success"
              }`}
            />
            <span className="text-content-primary font-medium">
              v{currentVersion}
            </span>
          </button>
        }
      />

      {/* Main settings section list */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <LanguageSelectionSection />
        <AppearanceSection />

        <SystemStartupSection
          isAutoStart={isAutoStart}
          isAlwaysOnTop={isAlwaysOnTop}
          isNotificationsEnabled={isNotificationsEnabled}
          isAutoTrimEnabled={isAutoTrimEnabled}
          onToggleAutoStart={handleToggleAutoStart}
          onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
          onToggleNotifications={handleToggleNotifications}
          onToggleAutoTrim={toggleAutoTrim}
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

      {/* Clear cache confirm dialog */}
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
