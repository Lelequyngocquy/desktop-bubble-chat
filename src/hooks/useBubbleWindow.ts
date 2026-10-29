import { useState, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMessengerStore, selectAccountByIndex } from "../stores/messengerStore";
import { useWindowStore } from "../stores/windowStore";
import { messengerService } from "../services/messenger";
import { getAccountThemeColor } from "../constants/accountColors";

/**
 * Custom hook dedicated to the floating Bubble window lifecycle and interactions.
 */
export function useBubbleWindow() {
  const [accountIndex, setAccountIndex] = useState<number>(1);
  const { isChatOpen, setIsChatOpen } = useWindowStore();

  useEffect(() => {
    // 1. Resolve account index from URL query parameter
    const params = new URLSearchParams(window.location.search);
    const accountParam = params.get("account");
    let resolvedIndex = accountParam ? parseInt(accountParam, 10) : 1;

    // 2. Resolve account index from Tauri window label (e.g. "bubble-02")
    try {
      const label = getCurrentWindow().label;
      const match = label.match(/bubble-(\d+)/);
      if (match) {
        resolvedIndex = parseInt(match[1], 10);
      }
    } catch {}

    setAccountIndex(resolvedIndex);
  }, []);

  const currentAccount = useMessengerStore((state) => selectAccountByIndex(state, accountIndex));
  const unreadCount = currentAccount?.unreadCount || 0;
  const themeColor = getAccountThemeColor(accountIndex);

  const handleBubbleClick = useCallback(async () => {
    try {
      const isOpen = await messengerService.toggleChat(accountIndex);
      setIsChatOpen(isOpen);
    } catch {
      // Dev / browser preview fallback
      setIsChatOpen(!isChatOpen);
    }
  }, [accountIndex, isChatOpen, setIsChatOpen]);

  return {
    accountIndex,
    currentAccount,
    unreadCount,
    themeColor,
    isChatOpen,
    handleBubbleClick,
  };
}
