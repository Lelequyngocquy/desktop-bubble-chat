import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore } from "../stores/messengerStore";
import { useWindowStore } from "../stores/windowStore";
import { messengerService, messengerAuthManager } from "../services/messenger";
import { systemService } from "../services/systemService";
import {
  IPC_EVENTS,
  MessengerAuthStatusPayload,
  MessengerAuthClosedPayload,
  BadgeUpdatedPayload,
  BubbleVisibilityPayload,
  ChatVisibilityPayload,
} from "../types/ipc";

/**
 * Global background listener synchronizing Tauri desktop IPC events with Domain Stores.
 * Mounts once on app startup and maintains event lifecycle cleanly without re-render loops.
 */
export const useUnreadListener = () => {
  useEffect(() => {
    // Chỉ kích hoạt bộ lắng nghe IPC toàn cục trên cửa sổ chính (main), tránh nhân bản listener trong các bubble window
    if (
      typeof window !== "undefined" &&
      (window.location.search.includes("window=bubble") ||
        window.location.href.includes("bubble"))
    ) {
      return;
    }

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
                useMessengerStore.getState().updateAccount(acc.id, {
                  isConnected: true,
                  isAuthenticating: false,
                  lastSynced: "Đồng bộ vừa xong",
                });
                // Khôi phục hiển thị Bubble nếu tài khoản này đang có cấu hình bật
                if (acc.isBubbleVisible) {
                  messengerService.toggleBubble(true, acc.index).catch(() => {});
                }
              }
            })
            .catch(() => {});
        });

        // 2. Unread Badge Count event (Scoped theo accountIndex)
        unlistenBadge = await listen<number | BadgeUpdatedPayload>(IPC_EVENTS.BADGE_UPDATED, (event) => {
          if (typeof event.payload === "object" && event.payload !== null && "count" in event.payload) {
            const { accountIndex, count } = event.payload;
            useMessengerStore.getState().setAccountUnreadCount(accountIndex, count);
          } else if (typeof event.payload === "number") {
            const activeIndex = useMessengerStore.getState().activeAccountIndex;
            useMessengerStore.getState().setAccountUnreadCount(activeIndex, event.payload);
          }
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
                isBubbleVisible: isConnected ? true : existingAccount.isBubbleVisible,
                lastSynced: isConnected ? "Đồng bộ vừa xong" : "Chưa kết nối",
              });
            }

            if (isConnected) {
              messengerAuthManager.cancelAuth(accountId);
              store.setAccountBubbleVisibility(accountIdx, true);
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
            messengerAuthManager.cancelAuth(accountId);
            useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
          }
        );

        // 5. Floating bubble visibility change (Scoped theo accountIndex - SSOT)
        unlistenBubble = await listen<boolean | BubbleVisibilityPayload>(IPC_EVENTS.BUBBLE_VISIBILITY_CHANGED, (event) => {
          if (typeof event.payload === "object" && event.payload !== null && "visible" in event.payload) {
            const { accountIndex, visible } = event.payload;
            const idx = accountIndex || 1;
            useMessengerStore.getState().setAccountBubbleVisibility(idx, visible);
          } else if (typeof event.payload === "boolean") {
            const activeIdx = useMessengerStore.getState().activeAccountIndex;
            useMessengerStore.getState().setAccountBubbleVisibility(activeIdx, event.payload);
          }
        });

        // 6. Messenger Chat popup visibility change (Scoped theo accountIndex - SSOT)
        unlistenChat = await listen<boolean | ChatVisibilityPayload>(IPC_EVENTS.CHAT_VISIBILITY_CHANGED, (event) => {
          if (typeof event.payload === "object" && event.payload !== null && "visible" in event.payload) {
            const { accountIndex, visible } = event.payload;
            if (visible) {
              useWindowStore.getState().setOpenChatAccountIndex(accountIndex || 1);
            } else {
              useWindowStore.getState().setOpenChatAccountIndex(null);
            }
          } else if (typeof event.payload === "boolean") {
            if (event.payload) {
              const activeIdx = useMessengerStore.getState().activeAccountIndex;
              useWindowStore.getState().setOpenChatAccountIndex(activeIdx);
            } else {
              useWindowStore.getState().setOpenChatAccountIndex(null);
            }
          }
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
  }, []); // Run once on app mount

  // Helper for testing unread count
  const simulateUnread = async (count: number) => {
    const activeIndex = useMessengerStore.getState().activeAccountIndex;
    useMessengerStore.getState().setAccountUnreadCount(activeIndex, count);
    await systemService.setBadgeCount(count);
  };

  return { simulateUnread };
};
