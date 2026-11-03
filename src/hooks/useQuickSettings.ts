import { useState, useEffect, useCallback, useMemo } from "react";
import { useSettingsStore, BubbleSize, BUBBLE_SIZE_PIXELS } from "../stores/settingsStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
  selectAreAnyBubblesVisible,
  selectAreAllBubblesVisible,
  selectVisibleBubbleCount,
  selectAreAnySoundsEnabled,
  selectAreAllSoundsEnabled,
  selectSoundEnabledCount,
  messengerService,
} from "../platforms/messenger";
import { systemService } from "../services/systemService";
import { notificationService } from "../services/notificationService";

export function useQuickSettings() {
  const isAlwaysOnTop = useSettingsStore((s) => s.isAlwaysOnTop);
  const toggleAlwaysOnTop = useSettingsStore((s) => s.toggleAlwaysOnTop);
  const setNotificationsEnabled = useSettingsStore((s) => s.setNotificationsEnabled);
  const isAutoStart = useSettingsStore((s) => s.isAutoStart);
  const setAutoStart = useSettingsStore((s) => s.setAutoStart);
  const toggleAutoStart = useSettingsStore((s) => s.toggleAutoStart);
  const bubbleSize = useSettingsStore((s) => s.bubbleSize);
  const setBubbleSize = useSettingsStore((s) => s.setBubbleSize);

  // SSOT: Trạng thái Bong bóng nổi được dẫn xuất trực tiếp từ Domain Accounts
  const isConnected = useMessengerStore(selectIsMessengerConnected);
  const accounts = useMessengerStore((s) => s.accounts);
  const connectedAccounts = useMemo(() => accounts.filter((a) => a.isConnected), [accounts]);
  const areAnyBubblesVisible = useMessengerStore(selectAreAnyBubblesVisible);
  const areAllBubblesVisible = useMessengerStore(selectAreAllBubblesVisible);
  const visibleBubbleCount = useMessengerStore(selectVisibleBubbleCount);
  const setAllBubblesVisibility = useMessengerStore((s) => s.setAllBubblesVisibility);

  // SSOT: Trạng thái Âm thanh thông báo được dẫn xuất trực tiếp từ Domain Accounts (Giai đoạn 4)
  const areAnySoundsEnabled = useMessengerStore(selectAreAnySoundsEnabled);
  const areAllSoundsEnabled = useMessengerStore(selectAreAllSoundsEnabled);
  const soundEnabledCount = useMessengerStore(selectSoundEnabledCount);
  const setAllAccountsSound = useMessengerStore((s) => s.setAllAccountsSound);

  const [warningMessage, setWarningMessage] = useState<string | null>(null);

  // Sync real Windows Registry autostart setting, Always-on-top, Bubble size, and Sound state on mount
  useEffect(() => {
    systemService.isAutostartEnabled().then((enabled) => {
      setAutoStart(enabled);
    });
    systemService.setAlwaysOnTop(isAlwaysOnTop);
    systemService.setBubbleConfig(BUBBLE_SIZE_PIXELS[bubbleSize]);
    setNotificationsEnabled(areAnySoundsEnabled);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleSetBubbleSize = useCallback(
    async (size: BubbleSize) => {
      setBubbleSize(size);
      await systemService.setBubbleConfig(BUBBLE_SIZE_PIXELS[size]);
    },
    [setBubbleSize]
  );

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

  const handleToggleAlwaysOnTop = useCallback(async () => {
    const nextVal = !isAlwaysOnTop;
    toggleAlwaysOnTop();
    await systemService.setAlwaysOnTop(nextVal);
  }, [isAlwaysOnTop, toggleAlwaysOnTop]);

  // Master Toggle: Bật/Tắt toàn bộ âm thanh thông báo của các tài khoản (SSOT)
  const handleToggleNotifications = useCallback(async () => {
    // Nếu đang có bất kỳ tài khoản nào bật âm thanh -> Tắt tất cả. Ngược lại -> Bật tất cả
    const nextVal = !areAnySoundsEnabled;
    setAllAccountsSound(nextVal);
    setNotificationsEnabled(nextVal);
    if (nextVal) {
      await notificationService.requestPermission();
    }
  }, [areAnySoundsEnabled, setAllAccountsSound, setNotificationsEnabled]);

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

  const notificationsDescription = !isConnected
    ? (areAnySoundsEnabled ? "Đang bật âm thanh thông báo" : "Đang tắt âm thanh thông báo")
    : areAnySoundsEnabled
      ? `${soundEnabledCount}/${connectedAccounts.length} tài khoản đang bật âm thanh`
      : "Đang tắt âm thanh tất cả tài khoản";

  return {
    isBubbleEnabled: areAnyBubblesVisible,
    areAllBubblesVisible,
    visibleBubbleCount,
    bubbleDescription,
    isAlwaysOnTop,
    isNotificationsEnabled: areAnySoundsEnabled,
    areAnySoundsEnabled,
    areAllSoundsEnabled,
    soundEnabledCount,
    notificationsDescription,
    isAutoStart,
    bubbleSize,
    isConnected,
    warningMessage,
    clearWarning: () => setWarningMessage(null),
    handleToggleBubble,
    handleSetBubbleSize,
    handleToggleAlwaysOnTop,
    handleToggleNotifications,
    handleToggleAutoStart,
  };
}
