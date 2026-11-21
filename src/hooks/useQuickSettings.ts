import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore, BubbleSize, BUBBLE_SIZE_PIXELS } from "../stores/settingsStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
  selectConnectedAccountCount,
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
  const { t } = useTranslation("dashboard");
  const isAlwaysOnTop = useSettingsStore((s) => s.isAlwaysOnTop);
  const toggleAlwaysOnTop = useSettingsStore((s) => s.toggleAlwaysOnTop);
  const setNotificationsEnabled = useSettingsStore((s) => s.setNotificationsEnabled);
  const isAutoStart = useSettingsStore((s) => s.isAutoStart);
  const toggleAutoStart = useSettingsStore((s) => s.toggleAutoStart);
  const bubbleSize = useSettingsStore((s) => s.bubbleSize);
  const setBubbleSize = useSettingsStore((s) => s.setBubbleSize);
  const isConnected = useMessengerStore(selectIsMessengerConnected);
  const connectedCount = useMessengerStore(selectConnectedAccountCount);
  const areAnyBubblesVisible = useMessengerStore(selectAreAnyBubblesVisible);
  const areAllBubblesVisible = useMessengerStore(selectAreAllBubblesVisible);
  const visibleBubbleCount = useMessengerStore(selectVisibleBubbleCount);
  const areAnySoundsEnabled = useMessengerStore(selectAreAnySoundsEnabled);
  const areAllSoundsEnabled = useMessengerStore(selectAreAllSoundsEnabled);
  const soundEnabledCount = useMessengerStore(selectSoundEnabledCount);
  const { setAllBubblesVisibility, setAllAccountsSound } = useMessengerStore.getState();
  const [warningMessage, setWarningMessage] = useState<string | null>(null);
  const warningTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => {
    return () => {
      clearTimeout(warningTimerRef.current);
    };
  }, []);
  const clearWarning = useCallback(() => {
    clearTimeout(warningTimerRef.current);
    setWarningMessage(null);
  }, []);
  const handleSetBubbleSize = useCallback(
    async (size: BubbleSize) => {
      setBubbleSize(size);
      await systemService.setBubbleConfig(BUBBLE_SIZE_PIXELS[size]);
    },
    [setBubbleSize],
  );
  const handleToggleBubble = useCallback(async () => {
    if (!isConnected && !areAnyBubblesVisible) {
      setWarningMessage(t("warning_connect_first"));
      clearTimeout(warningTimerRef.current);
      warningTimerRef.current = setTimeout(() => setWarningMessage(null), 3500);
      return;
    }
    const nextVal = !areAnyBubblesVisible;
    setAllBubblesVisibility(nextVal);
    try {
      const targetIndices = useMessengerStore.getState().accounts.filter((a) => a.isConnected).map((a) => a.index);
      await messengerService.toggleAllBubbles(nextVal, targetIndices.length > 0 ? targetIndices : [1]);
    } catch (err) {
      console.warn("[useQuickSettings] Failed to toggle all bubbles visibility:", err);
    }
  }, [isConnected, areAnyBubblesVisible, setAllBubblesVisibility, t]);
  const handleToggleAlwaysOnTop = useCallback(async () => {
    const nextVal = !isAlwaysOnTop;
    toggleAlwaysOnTop();
    await systemService.setAlwaysOnTop(nextVal);
  }, [isAlwaysOnTop, toggleAlwaysOnTop]);
  const handleToggleNotifications = useCallback(async () => {
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
    ? t("bubble_desc_need_connect")
    : areAnyBubblesVisible
      ? t("bubble_desc_visible_ratio", {
          visible: visibleBubbleCount,
          total: connectedCount,
        })
      : t("bubble_desc_all_hidden");
  const notificationsDescription = !isConnected
    ? (areAnySoundsEnabled ? t("notif_desc_sound_on") : t("notif_desc_sound_off"))
    : areAnySoundsEnabled
      ? t("notif_desc_accounts_on", {
          count: soundEnabledCount,
          total: connectedCount,
        })
      : t("notif_desc_accounts_off");
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
    clearWarning,
    handleToggleBubble,
    handleSetBubbleSize,
    handleToggleAlwaysOnTop,
    handleToggleNotifications,
    handleToggleAutoStart,
  };
}
