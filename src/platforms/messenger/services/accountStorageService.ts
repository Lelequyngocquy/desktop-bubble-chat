import {
  MessengerAccount,
  createMessengerAccount,
  formatMessengerName,
  formatMessengerAvatarText,
} from "../types";
import { getAccountThemeColor } from "../../../constants/design";

const STORAGE_KEY = "desktop_bubble_chat_messenger_accounts";

/**
 * Service handling persistent storage of Messenger accounts in localStorage.
 * Ensures data hygiene (e.g. resetting transient authentication flags, maintaining canonical naming).
 */
export const accountStorageService = {
  load(): MessengerAccount[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw !== null) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
          return parsed.map((a: MessengerAccount) => {
            const shouldRestore =
              typeof a.shouldRestoreBubble === "boolean"
                ? a.shouldRestoreBubble
                : typeof a.isBubbleVisible === "boolean"
                ? a.isBubbleVisible
                : false;

            return {
              ...a,
              // Bất biến SSOT: Tên tài khoản luôn luôn là "Messenger 01", "Messenger 02",... tuyệt đối không bị ghi đè bởi chuỗi số định danh UID
              name: formatMessengerName(a.index),
              avatarText: formatMessengerAvatarText(a.index),
              color: getAccountThemeColor(a.index).hex,
              isAuthenticating: false, // Invariant: Never persist in-flight authentication state
              // Khởi động app với trạng thái bong bóng người dùng đã thiết lập trước đó
              isBubbleVisible: shouldRestore,
              shouldRestoreBubble: shouldRestore,
              soundEnabled: typeof a.soundEnabled === "boolean" ? a.soundEnabled : true,
              hideMessageContent: typeof a.hideMessageContent === "boolean" ? a.hideMessageContent : false,
            };
          });
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
        // Bảo vệ tên chuẩn mực "Messenger 01", "Messenger 02",... khi lưu trữ
        name: formatMessengerName(a.index),
        avatarText: formatMessengerAvatarText(a.index),
        isAuthenticating: false,
        shouldRestoreBubble:
          typeof a.shouldRestoreBubble === "boolean"
            ? a.shouldRestoreBubble
            : a.isBubbleVisible,
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
