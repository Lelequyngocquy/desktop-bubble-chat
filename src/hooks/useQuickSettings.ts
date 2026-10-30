import { useState, useEffect, useCallback } from "react";
import { useSettingsStore } from "../stores/settingsStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
  selectConnectedAccounts,
  selectAreAnyBubblesVisible,
  selectAreAllBubblesVisible,
  selectVisibleBubbleCount,
} from "../stores/messengerStore";
import { systemService } from "../services/systemService";
import { messengerService } from "../services/messenger";

export function useQuickSettings() {
  const isAlwaysOnTop = useSettingsStore((s) => s.isAlwaysOnTop);
  const toggleAlwaysOnTop = useSettingsStore((s) => s.toggleAlwaysOnTop);
  const isNotificationsEnabled = useSettingsStore((s) => s.isNotificationsEnabled);
  const toggleNotifications = useSettingsStore((s) => s.toggleNotifications);
  const isAutoStart = useSettingsStore((s) => s.isAutoStart);
  const setAutoStart = useSettingsStore((s) => s.setAutoStart);
  const toggleAutoStart = useSettingsStore((s) => s.toggleAutoStart);

  // SSOT: Trạng thái Bong bóng nổi được dẫn xuất trực tiếp từ Domain Accounts
  const isConnected = useMessengerStore(selectIsMessengerConnected);
  const connectedAccounts = useMessengerStore(selectConnectedAccounts);
  const areAnyBubblesVisible = useMessengerStore(selectAreAnyBubblesVisible);
  const areAllBubblesVisible = useMessengerStore(selectAreAllBubblesVisible);
  const visibleBubbleCount = useMessengerStore(selectVisibleBubbleCount);
  const setAllBubblesVisibility = useMessengerStore((s) => s.setAllBubblesVisibility);

  const [warningMessage, setWarningMessage] = useState<string | null>(null);

  // Sync real Windows Registry autostart setting on mount
  useEffect(() => {
    systemService.isAutostartEnabled().then((enabled) => {
      setAutoStart(enabled);
    });
  }, [setAutoStart]);

  // Master Toggle: Bật/Tắt toàn bộ bong bóng của các tài khoản đã kết nối
  const handleToggleBubble = useCallback(async () => {
    if (!isConnected && !areAnyBubblesVisible) {
      setWarningMessage("Vui lòng kết nối Messenger trước khi bật bong bóng chat!");
      setTimeout(() => setWarningMessage(null), 3500);
      return;
    }

    // Nếu đang có bất kỳ bubble nào bật -> Tắt tất cả. Ngược lại -> Bật tất cả
    const nextVal = !areAnyBubblesVisible;
    setAllBubblesVisibility(nextVal);

    try {
      const targetIndices = connectedAccounts.map((a) => a.index);
      await messengerService.toggleAllBubbles(nextVal, targetIndices.length > 0 ? targetIndices : [1]);
    } catch (err) {
      console.warn("[useQuickSettings] Failed to toggle all bubbles visibility:", err);
    }
  }, [isConnected, areAnyBubblesVisible, setAllBubblesVisibility, connectedAccounts]);

  const handleToggleAutoStart = useCallback(async () => {
    const nextVal = !isAutoStart;
    toggleAutoStart();
    await systemService.toggleAutostart(nextVal);
  }, [isAutoStart, toggleAutoStart]);

  const bubbleDescription = !isConnected
    ? "Cần kết nối Messenger để hiển thị bong bóng"
    : areAnyBubblesVisible
      ? `${visibleBubbleCount}/${connectedAccounts.length} bong bóng đang hiển thị`
      : "Đang ẩn tất cả bong bóng nổi";

  return {
    isBubbleEnabled: areAnyBubblesVisible,
    areAllBubblesVisible,
    visibleBubbleCount,
    bubbleDescription,
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
