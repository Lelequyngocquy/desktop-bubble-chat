import { MessengerAccount, createMessengerAccount } from "../../types/account";

const STORAGE_KEY = "desktop_bubble_chat_messenger_accounts";

/**
 * Service handling persistent storage of Messenger accounts in localStorage.
 * Ensures data hygiene (e.g. resetting transient authentication flags).
 */
export const accountStorageService = {
  load(): MessengerAccount[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw !== null) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
          return parsed.map((a: MessengerAccount) => ({
            ...a,
            isAuthenticating: false, // Invariant: Never persist in-flight authentication state
          }));
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
        isAuthenticating: false,
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
