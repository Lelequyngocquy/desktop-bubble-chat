import { useState, useEffect, useCallback } from "react";
import { useSettingsStore } from "../stores/settingsStore";
import { useMessengerStore, selectIsMessengerConnected } from "../stores/messengerStore";
import { systemService } from "../services/systemService";
import { messengerService } from "../services/messenger";

export function useQuickSettings() {
  const {
    isBubbleEnabled,
    toggleBubble,
    isAlwaysOnTop,
    toggleAlwaysOnTop,
    isNotificationsEnabled,
    toggleNotifications,
    isAutoStart,
    setAutoStart,
    toggleAutoStart,
  } = useSettingsStore();

  const isConnected = useMessengerStore(selectIsMessengerConnected);
  const activeAccountIndex = useMessengerStore((s) => s.activeAccountIndex);

  const [warningMessage, setWarningMessage] = useState<string | null>(null);

  // Sync real Windows Registry autostart setting on mount
  useEffect(() => {
    systemService.isAutostartEnabled().then((enabled) => {
      setAutoStart(enabled);
    });
  }, [setAutoStart]);

  const handleToggleBubble = useCallback(async () => {
    if (!isConnected && !isBubbleEnabled) {
      setWarningMessage("Vui lòng kết nối Messenger trước khi bật bong bóng chat!");
      setTimeout(() => setWarningMessage(null), 3500);
      return;
    }

    const nextVal = !isBubbleEnabled;
    toggleBubble();

    try {
      await messengerService.toggleBubble(nextVal, activeAccountIndex || 1);
    } catch (err) {
      console.warn("[useQuickSettings] Failed to toggle bubble visibility:", err);
    }
  }, [isConnected, isBubbleEnabled, toggleBubble, activeAccountIndex]);

  const handleToggleAutoStart = useCallback(async () => {
    const nextVal = !isAutoStart;
    toggleAutoStart();
    await systemService.toggleAutostart(nextVal);
  }, [isAutoStart, toggleAutoStart]);

  return {
    isBubbleEnabled,
    isAlwaysOnTop,
    isNotificationsEnabled,
    isAutoStart,
    isConnected,
    warningMessage,
    clearWarning: () => setWarningMessage(null),
    handleToggleBubble,
    handleToggleAlwaysOnTop: toggleAlwaysOnTop,
    handleToggleNotifications: toggleNotifications,
    handleToggleAutoStart,
  };
}
