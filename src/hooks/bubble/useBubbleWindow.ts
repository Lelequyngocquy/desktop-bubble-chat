import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore, selectAccountByIndex } from "../../stores/messengerStore";
import { useWindowStore, selectIsAccountChatOpen } from "../../stores/windowStore";
import { getAccountThemeColor } from "../../constants/accountColors";
import { IPC_EVENTS, ChatVisibilityPayload } from "../../types/ipc";

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

  const isChatOpen = useWindowStore((s) => selectIsAccountChatOpen(s, accountIndex));
  const setOpenChatAccountIndex = useWindowStore((s) => s.setOpenChatAccountIndex);

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

        if (visible && targetAccount !== undefined) {
          setOpenChatAccountIndex(targetAccount);
        } else if (!visible) {
          if (targetAccount === undefined || targetAccount === accountIndex) {
            setOpenChatAccountIndex(null);
          }
        }
      }
    ).then((un) => {
      unlisten = un;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [accountIndex, setOpenChatAccountIndex]);

  const currentAccount = useMessengerStore((state) => selectAccountByIndex(state, accountIndex));
  const unreadCount = currentAccount?.unreadCount || 0;
  const themeColor = getAccountThemeColor(accountIndex);

  return {
    accountIndex,
    currentAccount,
    unreadCount,
    themeColor,
    isChatOpen,
  };
}
