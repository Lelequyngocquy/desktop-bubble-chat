import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore } from "../stores/messengerStore";
import { useWindowStore } from "../stores/windowStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useConversationStore } from "../stores/conversationStore";
import { messengerService, messengerAuthManager } from "../services/messenger";
import { systemService } from "../services/systemService";
import { soundService } from "../services/soundService";
import { notificationService } from "../services/notificationService";
import {
  IPC_EVENTS,
  MessengerAuthStatusPayload,
  MessengerAuthClosedPayload,
  BadgeUpdatedPayload,
  BubbleVisibilityPayload,
  ChatVisibilityPayload,
  IncomingMessagePayload,
  RecentConversationsSyncedPayload,
} from "../types/ipc";

/**
 * Global background listener synchronizing Tauri desktop IPC events with Domain Stores.
 * Mounts once on app startup and maintains event lifecycle cleanly without re-render loops.
 */
export const useUnreadListener = () => {
  const prevUnreadMap = useRef<Map<number, number>>(new Map());
  const isInitialGracePeriod = useRef<boolean>(true);

  useEffect(() => {
    // Chỉ kích hoạt bộ lắng nghe IPC toàn cục trên cửa sổ chính (main), tránh nhân bản listener trong các bubble window
    if (
      typeof window !== "undefined" &&
      (window.location.search.includes("window=bubble") ||
        window.location.href.includes("bubble"))
    ) {
      return;
    }

    const graceTimer = setTimeout(() => {
      isInitialGracePeriod.current = false;
    }, 3500);

    let unlistenBadge: (() => void) | undefined;
    let unlistenAuth: (() => void) | undefined;
    let unlistenAuthClosed: (() => void) | undefined;
    let unlistenBubble: (() => void) | undefined;
    let unlistenChat: (() => void) | undefined;
    let unlistenNewMessage: (() => void) | undefined;
    let unlistenRecent: (() => void) | undefined;

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
              }
            })
            .catch(() => {});
        });

        // 2. Unread Badge Count event (Scoped theo accountIndex)
        unlistenBadge = await listen<number | BadgeUpdatedPayload>(IPC_EVENTS.BADGE_UPDATED, (event) => {
          let idx = 1;
          let count = 0;
          if (typeof event.payload === "object" && event.payload !== null && "count" in event.payload) {
            idx = event.payload.accountIndex || 1;
            count = event.payload.count;
          } else if (typeof event.payload === "number") {
            idx = useMessengerStore.getState().activeAccountIndex;
            count = event.payload;
          }

          useMessengerStore.getState().setAccountUnreadCount(idx, count);

          const prev = prevUnreadMap.current.get(idx) ?? 0;
          if (!isInitialGracePeriod.current && count > prev) {
            useMessengerStore.getState().triggerAccountPulse(idx);
            const isNotificationsEnabled = useSettingsStore.getState().isNotificationsEnabled;
            const openChatAccount = useWindowStore.getState().openChatAccountIndex;
            const isChatOpenForAccount = openChatAccount === idx;

            if (isNotificationsEnabled && !isChatOpenForAccount) {
              soundService.playBubblePop();
              const account = useMessengerStore.getState().accounts.find((a) => a.index === idx);
              const accountName = account ? account.name : `Messenger ${idx.toString().padStart(2, "0")}`;
              notificationService.sendNotification(`Tin nhắn mới từ ${accountName}`, {
                body: `Bạn có ${count} tin nhắn chưa đọc`,
              });
            }
          }
          prevUnreadMap.current.set(idx, count);
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

            // Chỉ tự động bật Bubble khi người dùng vừa chủ động hoàn tất đăng nhập qua Modal Auth
            if (isConnected && (existingAccount?.isAuthenticating || messengerAuthManager.hasSession(accountId))) {
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

        // 6. New Message Received event (Non-DOM real-time interception)
        unlistenNewMessage = await listen<IncomingMessagePayload>(
          IPC_EVENTS.NEW_MESSAGE_RECEIVED,
          (event) => {
            const msg = event.payload;
            if (!msg) return;

            useConversationStore.getState().upsertConversationFromMessage(msg);
            useMessengerStore.getState().triggerAccountPulse(msg.accountIndex);
            if (msg.avatarUrl) {
              useMessengerStore.getState().setAccountLatestAvatar(msg.accountIndex, msg.avatarUrl);
            }

            const isNotificationsEnabled = useSettingsStore.getState().isNotificationsEnabled;
            const openChatAccount = useWindowStore.getState().openChatAccountIndex;
            const isChatOpenForAccount = openChatAccount === msg.accountIndex;

            if (isNotificationsEnabled && !isChatOpenForAccount) {
              soundService.playBubblePop();
              notificationService.sendNotification(msg.senderName, {
                body: msg.body || "Đã gửi một tin nhắn",
                icon: msg.avatarUrl,
              });
            }
          }
        );

        // 7. Recent Conversations Synced event (Fallback extraction)
        unlistenRecent = await listen<RecentConversationsSyncedPayload>(
          IPC_EVENTS.RECENT_CONVERSATIONS_SYNCED,
          (event) => {
            const data = event.payload;
            if (data && data.conversations) {
              useConversationStore
                .getState()
                .syncRecentConversations(data.accountIndex || 1, data.conversations);
            }
          }
        );
      } catch (err) {
        console.warn("[useUnreadListener] Desktop IPC event listeners not available in preview:", err);
      }
    };

    setupListeners();

    return () => {
      clearTimeout(graceTimer);
      if (unlistenBadge) unlistenBadge();
      if (unlistenAuth) unlistenAuth();
      if (unlistenAuthClosed) unlistenAuthClosed();
      if (unlistenBubble) unlistenBubble();
      if (unlistenChat) unlistenChat();
      if (unlistenNewMessage) unlistenNewMessage();
      if (unlistenRecent) unlistenRecent();
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
