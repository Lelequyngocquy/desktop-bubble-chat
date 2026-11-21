import {
  MessengerAccount,
  MAX_MESSENGER_ACCOUNTS,
  createMessengerAccount,
  formatMessengerName,
  formatMessengerAvatarText,
} from "../types";
import { getAccountThemeColor } from "../../../constants/design";

const STORAGE_KEY = "desktop_bubble_chat_messenger_accounts";

function sanitizeStoredAccount(raw: unknown): MessengerAccount | null {
  if (!raw || typeof raw !== "object") return null;
  const candidate = raw as Partial<MessengerAccount>;
  const index = candidate.index;
  if (typeof index !== "number" || !Number.isInteger(index) || index < 1 || index > MAX_MESSENGER_ACCOUNTS) {
    return null;
  }
  const base = createMessengerAccount(index);
  const shouldRestore =
    typeof candidate.shouldRestoreBubble === "boolean"
      ? candidate.shouldRestoreBubble
      : typeof candidate.isBubbleVisible === "boolean"
        ? candidate.isBubbleVisible
        : false;
  return {
    ...base,
    id: typeof candidate.id === "string" && candidate.id.trim() ? candidate.id : base.id,
    name: formatMessengerName(index),
    avatarText: formatMessengerAvatarText(index),
    color: getAccountThemeColor(index).hex,
    isConnected: typeof candidate.isConnected === "boolean" ? candidate.isConnected : false,
    isAuthenticating: false,
    isBubbleVisible: shouldRestore,
    shouldRestoreBubble: shouldRestore,
    unreadCount: typeof candidate.unreadCount === "number" && Number.isFinite(candidate.unreadCount) && candidate.unreadCount >= 0 ? candidate.unreadCount : 0,
    soundEnabled: typeof candidate.soundEnabled === "boolean" ? candidate.soundEnabled : true,
    hideMessageContent: typeof candidate.hideMessageContent === "boolean" ? candidate.hideMessageContent : false,
    lastSynced: typeof candidate.lastSynced === "string" ? candidate.lastSynced : base.lastSynced,
    latestSenderAvatar: typeof candidate.latestSenderAvatar === "string" ? candidate.latestSenderAvatar : undefined,
  };
}

/**
 * Service handling persistent storage of Messenger accounts in localStorage.
 */
export const accountStorageService = {
  load(): MessengerAccount[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw !== null) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
          const seen = new Set<number>();
          const validAccounts: MessengerAccount[] = [];
          for (const item of parsed) {
            const sanitized = sanitizeStoredAccount(item);
            if (sanitized && !seen.has(sanitized.index)) {
              seen.add(sanitized.index);
              validAccounts.push(sanitized);
            }
          }
          if (validAccounts.length > 0) {
            return validAccounts.sort((a, b) => a.index - b.index);
          }
        }
      }
    } catch (err) {
      console.warn("[AccountStorageService] Failed to load accounts from storage:", err);
    }
    return [createMessengerAccount(1)];
  },

  save(accounts: MessengerAccount[]): void {
    try {
      const sanitized = accounts.map((a) => ({
        ...a,
        name: formatMessengerName(a.index),
        avatarText: formatMessengerAvatarText(a.index),
        isAuthenticating: false,
        shouldRestoreBubble: typeof a.shouldRestoreBubble === "boolean" ? a.shouldRestoreBubble : a.isBubbleVisible,
      }));
      localStorage.setItem(STORAGE_KEY, JSON.stringify(sanitized));
    } catch (err) {
      console.warn("[AccountStorageService] Failed to save accounts to storage:", err);
    }
  },

  clear(): void {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (err) {
      console.warn("[AccountStorageService] Failed to clear storage:", err);
    }
  },
};
