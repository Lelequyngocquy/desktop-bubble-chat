import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useWindowStore } from "../stores/windowStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useConversationStore } from "../stores/conversationStore";
import {
  useMessengerStore,
  messengerService,
  messengerAuthManager,
} from "../platforms/messenger";
import { systemService } from "../services/systemService";
import { windowService } from "../services/windowService";
import { safeListen } from "../utils/tauriEvents";
import {
  IPC_EVENTS,
  MessengerAuthStatusPayload,
  MessengerAuthClosedPayload,
  BadgeUpdatedPayload,
  BubbleVisibilityPayload,
  BubbleSpawnedPayload,
  ChatVisibilityPayload,
  IncomingMessagePayload,
  RecentConversationsSyncedPayload,
  ChatWindowResizedPayload,
} from "../types/ipc";

/**
 * Global background listener synchronizing Tauri desktop IPC events with Domain Stores.
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
    let disposed = false;
    let resizeDebounceTimer: ReturnType<typeof setTimeout> | undefined;
    const unlisteners: Array<() => void> = [];
    const graceTimer = setTimeout(() => {
      isInitialGracePeriod.current = false;
    }, 3500);
    const register = async <T>(eventName: string, onPayload: (payload: T) => void): Promise<boolean> => {
      if (disposed) return false;
      await safeListen<T>(
        eventName,
        (event) => {
          if (!disposed) onPayload(event.payload);
        },
        () => disposed,
        (fn) => unlisteners.push(fn),
      );
      return !disposed;
    };
    const setupListeners = async () => {
      try {
        const currentAccounts = useMessengerStore.getState().accounts;
        const initialHasConnected = currentAccounts.some((a) => a.isConnected);
        systemService.updateTrayMenu(initialHasConnected);
        messengerService
          .getAllUnreadCounts()
          .then((counts) => {
            if (!disposed && counts && typeof counts === "object") {
              Object.entries(counts).forEach(([idxStr, cnt]) => {
                const idx = parseInt(idxStr, 10);
                if (!isNaN(idx)) {
                  useMessengerStore.getState().setAccountUnreadCount(idx, cnt);
                  prevUnreadMap.current.set(idx, cnt);
                  if (cnt <= 0) {
                    useConversationStore.getState().clearAccountConversations(idx);
                  }
                }
              });
            }
          })
          .catch(() => {});
        messengerService
          .getRecentMessages()
          .then((messages) => {
            if (!disposed && messages && messages.length > 0) {
              useConversationStore.getState().syncRecentIncomingMessages(messages);
            }
          })
          .catch(() => {});
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
          const prev = prevUnreadMap.current.get(idx) ?? 0;
          if (!isInitialGracePeriod.current && count > prev) {
            useMessengerStore.getState().triggerAccountPulse(idx);
          }
          prevUnreadMap.current.set(idx, count);
        }))) return;
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
            messengerService.toggleBubble(true, accountIdx).catch(() => {});
          }
        }))) return;
        if (!(await register<MessengerAuthClosedPayload>(IPC_EVENTS.MESSENGER_AUTH_CLOSED, (payload) => {
          const label = payload.label || "";
          let idx = 1;
          const match = label.match(/chat-messenger-(\d+)/);
          if (match) {
            idx = parseInt(match[1], 10);
          }
          const accountId = `messenger-${idx.toString().padStart(2, "0")}`;
          messengerAuthManager.cancelAuth(accountId);
          useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
        }))) return;
        if (!(await register<boolean | BubbleVisibilityPayload>(IPC_EVENTS.BUBBLE_VISIBILITY_CHANGED, (payload) => {
          if (typeof payload === "object" && payload !== null && "visible" in payload) {
            const { accountIndex, visible } = payload;
            const idx = accountIndex || 1;
            useMessengerStore.getState().setAccountBubbleVisibility(idx, visible);
          } else if (typeof payload === "boolean") {
            const activeIdx = useMessengerStore.getState().activeAccountIndex;
            useMessengerStore.getState().setAccountBubbleVisibility(activeIdx, payload);
          }
        }))) return;
        if (!(await register<boolean | ChatVisibilityPayload>(IPC_EVENTS.CHAT_VISIBILITY_CHANGED, (payload) => {
          const windowStore = useWindowStore.getState();
          if (typeof payload === "object" && payload !== null && "visible" in payload) {
            const targetIdx = payload.accountIndex || 1;
            if (payload.visible) {
              windowStore.setOpenChatAccountIndex(targetIdx);
            } else if (windowStore.openChatAccountIndex === targetIdx) {
              windowStore.setOpenChatAccountIndex(null);
            }
          } else if (typeof payload === "boolean") {
            const activeIdx = useMessengerStore.getState().activeAccountIndex;
            if (payload) {
              windowStore.setOpenChatAccountIndex(activeIdx);
            } else if (windowStore.openChatAccountIndex === activeIdx) {
              windowStore.setOpenChatAccountIndex(null);
            }
          }
        }))) return;
        if (!(await register<IncomingMessagePayload>(IPC_EVENTS.NEW_MESSAGE_RECEIVED, (msg) => {
          if (!msg) return;
          useConversationStore.getState().upsertConversationFromMessage(msg);
          useMessengerStore.getState().triggerAccountPulse(msg.accountIndex);
          if (msg.avatarUrl) {
            useMessengerStore.getState().setAccountLatestAvatar(msg.accountIndex, msg.avatarUrl);
          }
        }))) return;
        if (!(await register<RecentConversationsSyncedPayload>(IPC_EVENTS.RECENT_CONVERSATIONS_SYNCED, (data) => {
          if (data && data.conversations) {
            useConversationStore
              .getState()
              .syncRecentConversations(data.accountIndex || 1, data.conversations);
          }
        }))) return;
        if (!(await register(IPC_EVENTS.SHOW_CLOSE_DIALOG, () => {
          const action = useSettingsStore.getState().closeWindowAction;
          if (action === "hide_to_tray") {
            windowService.hide();
          } else if (action === "exit_app") {
            windowService.exit();
          } else {
            useWindowStore.getState().setIsCloseDialogOpen(true);
          }
        }))) return;
        if (!(await register<BubbleSpawnedPayload>(IPC_EVENTS.BUBBLE_SPAWNED, (payload) => {
          if (payload) {
            useWindowStore.getState().setLastBubbleSpawn(payload);
          }
        }))) return;
        await register<ChatWindowResizedPayload>(IPC_EVENTS.CHAT_WINDOW_RESIZED, (data) => {
          if (data && data.accountIndex && data.width && data.height) {
            const currentSetting = useSettingsStore.getState().chatWindowSettings.find(
              (s) => s.accountIndex === data.accountIndex,
            );
            if (currentSetting && currentSetting.mode === "remember_custom") {
              clearTimeout(resizeDebounceTimer);
              resizeDebounceTimer = setTimeout(() => {
                if (!disposed) {
                  useSettingsStore.getState().updateChatWindowSize(data.accountIndex, data.width, data.height);
                }
              }, 500);
            }
          }
        });
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
      disposed = true;
      clearTimeout(graceTimer);
      clearTimeout(resizeDebounceTimer);
      unsubscribeStore();
      unlisteners.forEach((unsub) => unsub());
    };
  }, []);
  const simulateUnread = async (count: number) => {
    const activeIndex = useMessengerStore.getState().activeAccountIndex;
    useMessengerStore.getState().setAccountUnreadCount(activeIndex, count);
    await systemService.setBadgeCount(count);
  };
  return { simulateUnread };
};
