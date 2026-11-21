import { useConversationStore } from "../../stores/conversationStore";
import {
  useMessengerStore,
  messengerService,
  messengerAuthManager,
} from "../../platforms/messenger";
import {
  IPC_EVENTS,
  MessengerAuthStatusPayload,
  MessengerAuthClosedPayload,
  BadgeUpdatedPayload,
  BubbleVisibilityPayload,
  IncomingMessagePayload,
  RecentConversationsSyncedPayload,
} from "../../types/ipc";
import { UnreadListenerContext } from "./types";

/**
 * Hydrates initial unread counts/messages and registers Messenger domain IPC events.
 */
export async function registerMessengerEvents(ctx: UnreadListenerContext): Promise<boolean> {
  const { register, isDisposed, prevUnreadMap, isInitialGracePeriod } = ctx;
  messengerService
    .getAllUnreadCounts()
    .then((counts) => {
      if (!isDisposed() && counts && typeof counts === "object") {
        Object.entries(counts).forEach(([idxStr, cnt]) => {
          const idx = parseInt(idxStr, 10);
          if (!isNaN(idx)) {
            useMessengerStore.getState().setAccountUnreadCount(idx, cnt);
            prevUnreadMap.set(idx, cnt);
            if (cnt <= 0) {
              useConversationStore.getState().clearAccountConversations(idx);
            }
          }
        });
      }
    })
    .catch((err) => {
      console.warn("[useUnreadListener] Initial unread count sync skipped:", err);
    });
  messengerService
    .getRecentMessages()
    .then((messages) => {
      if (!isDisposed() && messages && messages.length > 0) {
        useConversationStore.getState().syncRecentIncomingMessages(messages);
      }
    })
    .catch((err) => {
      console.warn("[useUnreadListener] Initial recent messages sync skipped:", err);
    });
  if (!(await register<number | BadgeUpdatedPayload>(IPC_EVENTS.BADGE_UPDATED, (payload) => {
    let idx = 1;
    let count = 0;
    if (typeof payload === "object" && payload !== null && "count" in payload) {
      idx = payload.accountIndex || 1;
      count = payload.count;
    } else if (typeof payload === "number") {
      idx = useMessengerStore.getState().activeAccountIndex;
      count = payload;
    }
    useMessengerStore.getState().setAccountUnreadCount(idx, count);
    if (count <= 0) {
      useConversationStore.getState().clearAccountConversations(idx);
    }
    const prev = prevUnreadMap.get(idx) ?? 0;
    if (!isInitialGracePeriod.current && count > prev) {
      useMessengerStore.getState().triggerAccountPulse(idx);
    }
    prevUnreadMap.set(idx, count);
  }))) return false;
  if (!(await register<MessengerAuthStatusPayload>(IPC_EVENTS.MESSENGER_AUTH_STATUS, (payload) => {
    const isConnected = payload.connected;
    const accountIdx = payload.accountIndex || 1;
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
      messengerService.toggleBubble(true, accountIdx).catch((err) => {
        console.warn("[useUnreadListener] Failed to auto-show bubble after auth:", err);
      });
    }
  }))) return false;
  if (!(await register<MessengerAuthClosedPayload>(IPC_EVENTS.MESSENGER_AUTH_CLOSED, (payload) => {
    const label = payload.label || "";
    const match = label.match(/chat-messenger-(\d+)/);
    const idx = match ? parseInt(match[1], 10) : 1;
    const accountId = `messenger-${idx.toString().padStart(2, "0")}`;
    messengerAuthManager.cancelAuth(accountId);
    useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
  }))) return false;
  if (!(await register<boolean | BubbleVisibilityPayload>(IPC_EVENTS.BUBBLE_VISIBILITY_CHANGED, (payload) => {
    if (typeof payload === "object" && payload !== null && "visible" in payload) {
      useMessengerStore.getState().setAccountBubbleVisibility(payload.accountIndex || 1, payload.visible);
    } else if (typeof payload === "boolean") {
      const activeIdx = useMessengerStore.getState().activeAccountIndex;
      useMessengerStore.getState().setAccountBubbleVisibility(activeIdx, payload);
    }
  }))) return false;
  if (!(await register<IncomingMessagePayload>(IPC_EVENTS.NEW_MESSAGE_RECEIVED, (msg) => {
    if (!msg) return;
    useConversationStore.getState().upsertConversationFromMessage(msg);
    useMessengerStore.getState().triggerAccountPulse(msg.accountIndex);
    if (msg.avatarUrl) {
      useMessengerStore.getState().setAccountLatestAvatar(msg.accountIndex, msg.avatarUrl);
    }
  }))) return false;
  return register<RecentConversationsSyncedPayload>(IPC_EVENTS.RECENT_CONVERSATIONS_SYNCED, (data) => {
    if (data && data.conversations) {
      useConversationStore
        .getState()
        .syncRecentConversations(data.accountIndex || 1, data.conversations);
    }
  });
}
