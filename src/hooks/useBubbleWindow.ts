import { useState, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore, selectAccountByIndex } from "../stores/messengerStore";
import { useWindowStore } from "../stores/windowStore";
import { messengerService } from "../services/messenger";
import { getAccountThemeColor } from "../constants/accountColors";
import { IPC_EVENTS, ChatVisibilityPayload } from "../types/ipc";

/**
 * Custom hook dedicated to the floating Bubble window lifecycle and interactions.
 */
export function useBubbleWindow() {
  const [accountIndex, setAccountIndex] = useState<number>(() => {
    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search);
      const accountParam = params.get("account");
      if (accountParam) return parseInt(accountParam, 10);
      try {
        const label = getCurrentWindow().label;
        const match = label.match(/bubble-(\d+)/);
        if (match) return parseInt(match[1], 10);
      } catch {}
    }
    return 1;
  });
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

  // Lắng nghe sự kiện đóng/mở chat thời gian thực từ Rust backend cho đúng account này
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<boolean | ChatVisibilityPayload>(
      IPC_EVENTS.CHAT_VISIBILITY_CHANGED,
      (event) => {
        let visible = false;
        let targetAccount: number | undefined;

        if (
          typeof event.payload === "object" &&
          event.payload !== null &&
          "visible" in event.payload
        ) {
          visible = event.payload.visible;
          targetAccount = event.payload.accountIndex;
        } else if (typeof event.payload === "boolean") {
          visible = event.payload;
        }

        // Nếu có chat của tài khoản khác vừa mở -> Chat của mình chắc chắn phải đóng và mình phải bị tuck đi
        if (targetAccount !== undefined && targetAccount !== accountIndex && visible) {
          setIsChatOpen(false);
          return;
        }

        // Nếu sự kiện dành riêng cho tài khoản này -> Cập nhật trạng thái mở/đóng
        if (targetAccount === accountIndex) {
          setIsChatOpen(visible);
        } else if (targetAccount === undefined && !visible) {
          // Broadcast ẩn toàn bộ chat
          setIsChatOpen(false);
        }
      }
    ).then((un) => {
      unlisten = un;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [accountIndex, setIsChatOpen]);

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

  const handleBubbleHoverOpen = useCallback(async () => {
    try {
      await messengerService.openChat(accountIndex);
      setIsChatOpen(true);
    } catch {
      // Dev / browser preview fallback
      setIsChatOpen(true);
    }
  }, [accountIndex, setIsChatOpen]);

  return {
    accountIndex,
    currentAccount,
    unreadCount,
    themeColor,
    isChatOpen,
    handleBubbleClick,
    handleBubbleHoverOpen,
  };
}
