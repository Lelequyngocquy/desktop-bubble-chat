import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore } from "../stores/messengerStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useWindowStore } from "../stores/windowStore";
import { messengerService } from "../services/messenger";
import { systemService } from "../services/systemService";
import {
  IPC_EVENTS,
  MessengerAuthStatusPayload,
  MessengerAuthClosedPayload,
} from "../types/ipc";

/**
 * Global background listener synchronizing Tauri desktop IPC events with Domain Stores.
 */
export const useUnreadListener = () => {
  const { updateAccount, setAccountUnreadCount } = useMessengerStore();
  const { setBubbleEnabled } = useSettingsStore();
  const { setIsChatOpen } = useWindowStore();

  useEffect(() => {
    let unlistenBadge: (() => void) | undefined;
    let unlistenAuth: (() => void) | undefined;
    let unlistenAuthClosed: (() => void) | undefined;
    let unlistenBubble: (() => void) | undefined;
    let unlistenChat: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        // 1. Initial health-check: Verify and restore real authentication status for all accounts on boot
        const currentAccounts = useMessengerStore.getState().accounts;
        currentAccounts.forEach((acc) => {
          messengerService
            .checkAuth(acc.index)
            .then((res) => {
              if (res && res.connected) {
                updateAccount(acc.id, {
                  isConnected: true,
                  isAuthenticating: false,
                  lastSynced: "Đồng bộ vừa xong",
                });
                setBubbleEnabled(true);
                messengerService.toggleBubble(true, acc.index).catch(() => {});
              }
            })
            .catch(() => {});
        });

        // 2. Unread Badge Count event
        unlistenBadge = await listen<number>(IPC_EVENTS.BADGE_UPDATED, (event) => {
          const activeIndex = useMessengerStore.getState().activeAccountIndex;
          setAccountUnreadCount(activeIndex, event.payload);
        });

        // 3. Messenger real-time authentication status change
        unlistenAuth = await listen<MessengerAuthStatusPayload>(
          IPC_EVENTS.MESSENGER_AUTH_STATUS,
          (event) => {
            const isConnected = event.payload.connected;
            const accountIdx = event.payload.accountIndex || 1;
            const accountId = `messenger-${accountIdx.toString().padStart(2, "0")}`;
            const store = useMessengerStore.getState();

            const existingAccount = store.accounts.find((a) => a.id === accountId);
            if (existingAccount) {
              store.updateAccount(accountId, {
                isConnected,
                isAuthenticating: false,
                lastSynced: isConnected ? "Đồng bộ vừa xong" : "Chưa kết nối",
              });
            }

            if (isConnected) {
              setBubbleEnabled(true);
              messengerService.toggleBubble(true, accountIdx).catch(() => {});
            }
          }
        );

        // 4. Messenger Auth window closed by user
        unlistenAuthClosed = await listen<MessengerAuthClosedPayload>(
          IPC_EVENTS.MESSENGER_AUTH_CLOSED,
          (event) => {
            const label = event.payload.label || "";
            let idx = 1;
            const match = label.match(/chat-messenger-(\d+)/);
            if (match) {
              idx = parseInt(match[1], 10);
            }
            const accountId = `messenger-${idx.toString().padStart(2, "0")}`;
            useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
          }
        );

        // 5. Floating bubble visibility change (e.g. dismissed to tray)
        unlistenBubble = await listen<boolean>(IPC_EVENTS.BUBBLE_VISIBILITY_CHANGED, (event) => {
          setBubbleEnabled(event.payload);
        });

        // 6. Messenger Chat popup visibility change
        unlistenChat = await listen<boolean>(IPC_EVENTS.CHAT_VISIBILITY_CHANGED, (event) => {
          setIsChatOpen(event.payload);
        });
      } catch (err) {
        console.warn("[useUnreadListener] Desktop IPC event listeners not available in preview:", err);
      }
    };

    setupListeners();

    return () => {
      if (unlistenBadge) unlistenBadge();
      if (unlistenAuth) unlistenAuth();
      if (unlistenAuthClosed) unlistenAuthClosed();
      if (unlistenBubble) unlistenBubble();
      if (unlistenChat) unlistenChat();
    };
  }, [updateAccount, setAccountUnreadCount, setBubbleEnabled, setIsChatOpen]);

  // Helper for testing unread count
  const simulateUnread = async (count: number) => {
    const activeIndex = useMessengerStore.getState().activeAccountIndex;
    setAccountUnreadCount(activeIndex, count);
    await systemService.setBadgeCount(count);
  };

  return { simulateUnread };
};
