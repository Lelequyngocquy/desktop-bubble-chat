import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useWindowStore } from "../stores/windowStore";
import { useConversationStore } from "../stores/conversationStore";
import {
  useMessengerStore,
  messengerService,
  messengerAuthManager,
} from "../platforms/messenger";
import { systemService } from "../services/systemService";
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
 * Global background listener synchronizing Tauri desktop IPC events with Domain Stores
 */
export const useUnreadListener = () => {
  const prevUnreadMap = useRef<Map<number, number>>(new Map());
  const isInitialGracePeriod = useRef<boolean>(true);

  useEffect(() => {
    try {
      const currentWin = getCurrentWindow();
      if (currentWin && currentWin.label && currentWin.label !== "main") {
        return;
      }
    } catch {}
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
        const currentAccounts = useMessengerStore.getState().accounts;
        const initialHasConnected = currentAccounts.some((a) => a.isConnected);
        systemService.updateTrayMenu(initialHasConnected);
        currentAccounts.forEach((acc) => {
          if (acc.soundEnabled === false) {
            messengerService.setAccountMuted(acc.index, true).catch(() => {});
          }
          messengerService
            .checkAuth(acc.index)
            .then((res) => {
              if (res && res.connected) {
                useMessengerStore.getState().updateAccount(acc.id, {
                  isConnected: true,
                  isAuthenticating: false,
                  lastSynced: "",
                });
              }
            })
            .catch(() => {});
        });
        messengerService
          .getAllUnreadCounts()
          .then((counts) => {
            if (counts && typeof counts === "object") {
              Object.entries(counts).forEach(([idxStr, cnt]) => {
                const idx = parseInt(idxStr, 10);
                if (!isNaN(idx)) {
                  useMessengerStore.getState().setAccountUnreadCount(idx, cnt);
                  prevUnreadMap.current.set(idx, cnt);
                }
              });
            }
          })
          .catch(() => {});
        messengerService
          .getRecentMessages()
          .then((messages) => {
            if (messages && messages.length > 0) {
              useConversationStore.getState().syncRecentIncomingMessages(messages);
            }
          })
          .catch(() => {});
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
          }
          prevUnreadMap.current.set(idx, count);
        });
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
                lastSynced: "",
              });
            }
            if (isConnected && (existingAccount?.isAuthenticating || messengerAuthManager.hasSession(accountId))) {
              messengerAuthManager.cancelAuth(accountId);
              store.setAccountBubbleVisibility(accountIdx, true);
              messengerService.toggleBubble(true, accountIdx).catch(() => {});
            }
          }
        );
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
          }
        );
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
    const unsubscribeStore = useMessengerStore.subscribe((state, prevState) => {
      const wasConnected = prevState.accounts.some((a) => a.isConnected);
      const isConnected = state.accounts.some((a) => a.isConnected);
      if (wasConnected !== isConnected) {
        systemService.updateTrayMenu(isConnected);
      }
    });
    setupListeners();
    return () => {
      clearTimeout(graceTimer);
      unsubscribeStore();
      if (unlistenBadge) unlistenBadge();
      if (unlistenAuth) unlistenAuth();
      if (unlistenAuthClosed) unlistenAuthClosed();
      if (unlistenBubble) unlistenBubble();
      if (unlistenChat) unlistenChat();
      if (unlistenNewMessage) unlistenNewMessage();
      if (unlistenRecent) unlistenRecent();
    };
  }, []);

  const simulateUnread = async (count: number) => {
    const activeIndex = useMessengerStore.getState().activeAccountIndex;
    useMessengerStore.getState().setAccountUnreadCount(activeIndex, count);
    await systemService.setBadgeCount(count);
  };

  return { simulateUnread };
};