import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore, selectAccountByIndex } from "../../stores/messengerStore";
import { useWindowStore, selectIsAccountChatOpen } from "../../stores/windowStore";
import { useSettingsStore, BubbleSize } from "../../stores/settingsStore";
import { getAccountThemeColor } from "../../constants/accountColors";
import { IPC_EVENTS, ChatVisibilityPayload, BubbleConfigChangedPayload } from "../../types/ipc";

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

  const [directUnreadCount, setDirectUnreadCount] = useState<number>(0);
  const [directAvatarUrl, setDirectAvatarUrl] = useState<string | undefined>(undefined);
  const [directHasPulse, setDirectHasPulse] = useState<boolean>(false);

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

  // Đồng bộ kích thước Bong bóng thời gian thực với WebviewWindow và IPC Backend
  useEffect(() => {
    let unlistenResize: (() => void) | undefined;
    let unlistenConfig: (() => void) | undefined;

    const syncSizeFromWidth = (width: number) => {
      let resolved: BubbleSize = "md";
      if (width <= 50) resolved = "sm";
      else if (width <= 60) resolved = "md";
      else resolved = "lg";

      if (useSettingsStore.getState().bubbleSize !== resolved) {
        useSettingsStore.getState().setBubbleSize(resolved);
      }
    };

    try {
      const win = getCurrentWindow();
      win.innerSize().then((size) => {
        if (size?.width) syncSizeFromWidth(size.width);
      });

      win.onResized(({ payload: size }) => {
        if (size?.width) syncSizeFromWidth(size.width);
      }).then((un) => {
        unlistenResize = un;
      });
    } catch {}

    listen<BubbleConfigChangedPayload>(IPC_EVENTS.BUBBLE_CONFIG_CHANGED, (event) => {
      if (event.payload?.bubbleSize) {
        useSettingsStore.getState().setBubbleSize(event.payload.bubbleSize);
      }
    }).then((un) => {
      unlistenConfig = un;
    });

    return () => {
      if (unlistenResize) unlistenResize();
      if (unlistenConfig) unlistenConfig();
    };
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

  // Lắng nghe trực tiếp sự kiện BADGE_UPDATED và INCOMING_MESSAGE từ Rust Core
  useEffect(() => {
    let unlistenBadge: (() => void) | undefined;
    let unlistenMessage: (() => void) | undefined;

    listen<number | { accountIndex?: number; count: number }>(IPC_EVENTS.BADGE_UPDATED, (event) => {
      let idx = 1;
      let count = 0;
      if (typeof event.payload === "object" && event.payload !== null && "count" in event.payload) {
        idx = event.payload.accountIndex || 1;
        count = event.payload.count;
      } else if (typeof event.payload === "number") {
        count = event.payload;
      }
      if (idx === accountIndex) {
        setDirectUnreadCount(count);
      }
    }).then((un) => {
      unlistenBadge = un;
    });

    listen<{ accountIndex?: number; avatarUrl?: string }>(IPC_EVENTS.NEW_MESSAGE_RECEIVED, (event) => {
      const idx = event.payload?.accountIndex || 1;
      if (idx === accountIndex) {
        if (event.payload?.avatarUrl) {
          setDirectAvatarUrl(event.payload.avatarUrl);
        }
        setDirectHasPulse(true);
        setTimeout(() => setDirectHasPulse(false), 3000);
      }
    }).then((un) => {
      unlistenMessage = un;
    });

    return () => {
      if (unlistenBadge) unlistenBadge();
      if (unlistenMessage) unlistenMessage();
    };
  }, [accountIndex]);

  const currentAccount = useMessengerStore((state) => selectAccountByIndex(state, accountIndex));
  const unreadCount = directUnreadCount || currentAccount?.unreadCount || 0;
  const avatarUrl = directAvatarUrl || currentAccount?.latestSenderAvatar;
  const hasPulse = directHasPulse || currentAccount?.hasNewMessagePulse || false;
  const themeColor = getAccountThemeColor(accountIndex);

  return {
    accountIndex,
    currentAccount,
    unreadCount,
    avatarUrl,
    hasPulse,
    themeColor,
    isChatOpen,
  };
}
