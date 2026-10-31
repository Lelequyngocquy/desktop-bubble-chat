import { create } from "zustand";
import {
  MessengerAccount,
  ConnectedAccount,
  MAX_MESSENGER_ACCOUNTS,
  createMessengerAccount,
} from "../types/account";
import { accountStorageService } from "../services/storage/accountStorageService";
import { messengerAuthManager } from "../services/messenger/messengerAuthManager";

interface MessengerState {
  // --- SSOT Data ---
  accounts: MessengerAccount[];
  activeAccountIndex: number;

  // --- Actions ---
  setAccounts: (accounts: MessengerAccount[]) => void;
  setActiveAccountIndex: (index: number) => void;
  addAccount: () => MessengerAccount | null;
  removeAccount: (id: string) => void;
  updateAccount: (id: string, updates: Partial<MessengerAccount>) => void;
  disconnectAllAccounts: () => void;
  setAccountUnreadCount: (accountIndex: number, count: number) => void;
  setTotalUnreadCount: (count: number) => void;

  // --- Per-Bubble Visibility Actions (SSOT) ---
  setAccountBubbleVisibility: (accountIndex: number, visible: boolean) => void;
  toggleAccountBubbleVisibility: (accountIndex: number) => void;
  setAllBubblesVisibility: (visible: boolean) => void;

  // --- Visual Pulse & Sender Avatar Actions (Giai đoạn 3) ---
  triggerAccountPulse: (accountIndex: number) => void;
  clearAccountPulse: (accountIndex: number) => void;
  setAccountLatestAvatar: (accountIndex: number, avatarUrl?: string) => void;
}

const initialAccounts = accountStorageService.load();
const initialPrimary = initialAccounts.find((a) => a.isConnected);
const initialActiveIndex = initialPrimary ? initialPrimary.index : (initialAccounts[0]?.index || 1);

export const useMessengerStore = create<MessengerState>((set, get) => ({
  accounts: initialAccounts,
  activeAccountIndex: initialActiveIndex,

  setAccounts: (accounts) => {
    accountStorageService.save(accounts);
    const primary = accounts.find((a) => a.isConnected);
    set({
      accounts,
      activeAccountIndex: primary ? primary.index : (accounts[0]?.index || 1),
    });
  },

  setActiveAccountIndex: (index) => set({ activeAccountIndex: index }),

  addAccount: () => {
    const current = get().accounts;
    if (current.length >= MAX_MESSENGER_ACCOUNTS) {
      return null;
    }

    const usedIndices = new Set(current.map((a) => a.index));
    let nextIndex = 1;
    for (let i = 1; i <= MAX_MESSENGER_ACCOUNTS; i++) {
      if (!usedIndices.has(i)) {
        nextIndex = i;
        break; // Dừng ngay ở vị trí trống nhỏ nhất đầu tiên (ưu tiên tăng dần 01 -> 05)
      }
    }

    const newAccount = createMessengerAccount(nextIndex);
    const updated = [...current, newAccount].sort((a, b) => a.index - b.index);
    accountStorageService.save(updated);

    const currentActive = get().activeAccountIndex;
    const activeAccount = updated.find((a) => a.index === currentActive);
    const newActiveIndex = activeAccount?.isConnected
      ? currentActive
      : updated.find((a) => a.isConnected)?.index || currentActive;

    set({
      accounts: updated,
      activeAccountIndex: newActiveIndex,
    });
    return newAccount;
  },

  removeAccount: (id) => {
    messengerAuthManager.cancelAuth(id);
    const current = get().accounts;
    const updated = current.filter((a) => a.id !== id);
    accountStorageService.save(updated);

    const primary = updated.find((a) => a.isConnected);
    set({
      accounts: updated,
      activeAccountIndex: primary ? primary.index : (updated[0]?.index || 1),
    });
  },

  updateAccount: (id, updates) => {
    const current = get().accounts;
    const updated = current.map((a) => (a.id === id ? { ...a, ...updates } : a));
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  disconnectAllAccounts: () => {
    messengerAuthManager.cancelAll();
    const current = get().accounts;
    const updated = current.map((a) => ({
      ...a,
      isConnected: false,
      isAuthenticating: false,
      isBubbleVisible: false,
    }));
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  setAccountUnreadCount: (accountIndex, count) => {
    const current = get().accounts;
    const updated = current.map((a) => (a.index === accountIndex ? { ...a, unreadCount: count } : a));
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  setTotalUnreadCount: (count) => {
    const activeIndex = get().activeAccountIndex;
    const current = get().accounts;
    const updated = current.map((a) => (a.index === activeIndex ? { ...a, unreadCount: count } : a));
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  setAccountBubbleVisibility: (accountIndex, visible) => {
    const current = get().accounts;
    const updated = current.map((a) =>
      a.index === accountIndex
        ? { ...a, isBubbleVisible: visible, shouldRestoreBubble: visible }
        : a
    );
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  toggleAccountBubbleVisibility: (accountIndex) => {
    const current = get().accounts;
    const updated = current.map((a) => {
      if (a.index === accountIndex) {
        const nextVal = !a.isBubbleVisible;
        return { ...a, isBubbleVisible: nextVal, shouldRestoreBubble: nextVal };
      }
      return a;
    });
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  setAllBubblesVisibility: (visible) => {
    const current = get().accounts;
    const updated = current.map((a) =>
      a.isConnected
        ? { ...a, isBubbleVisible: visible, shouldRestoreBubble: visible }
        : a
    );
    accountStorageService.save(updated);
    set({ accounts: updated });
  },

  triggerAccountPulse: (accountIndex) => {
    const current = get().accounts;
    const updated = current.map((a) =>
      a.index === accountIndex ? { ...a, hasNewMessagePulse: true } : a
    );
    set({ accounts: updated });

    // Auto-clear pulse after 4s
    setTimeout(() => {
      get().clearAccountPulse(accountIndex);
    }, 4000);
  },

  clearAccountPulse: (accountIndex) => {
    const current = get().accounts;
    const updated = current.map((a) =>
      a.index === accountIndex ? { ...a, hasNewMessagePulse: false } : a
    );
    set({ accounts: updated });
  },

  setAccountLatestAvatar: (accountIndex, avatarUrl) => {
    const current = get().accounts;
    const updated = current.map((a) =>
      a.index === accountIndex ? { ...a, latestSenderAvatar: avatarUrl } : a
    );
    set({ accounts: updated });
  },
}));

// --- Pure Selectors (Computed from SSOT, guaranteed consistency) ---
export const selectIsMessengerConnected = (state: MessengerState): boolean =>
  state.accounts.some((a) => a.isConnected);

// Memoized selector: Trả về cùng instance mảng nếu state.accounts không thay đổi,
// bảo đảm an toàn tuyệt đối cho useSyncExternalStore trong React 18/19 (chống infinite re-render loop)
let lastAccountsRef: MessengerAccount[] | null = null;
let cachedConnectedAccounts: MessengerAccount[] = [];

export const selectConnectedAccounts = (state: MessengerState): MessengerAccount[] => {
  if (state.accounts === lastAccountsRef) {
    return cachedConnectedAccounts;
  }
  lastAccountsRef = state.accounts;
  cachedConnectedAccounts = state.accounts.filter((a) => a.isConnected);
  return cachedConnectedAccounts;
};

export const selectPrimaryAccount = (state: MessengerState): MessengerAccount | null =>
  state.accounts.find((a) => a.isConnected) || null;

export const selectConnectedAccountSummary = (state: MessengerState): ConnectedAccount | null => {
  return selectPrimaryAccount(state);
};

export const selectTotalUnreadCount = (state: MessengerState): number =>
  state.accounts.reduce((sum, a) => sum + (a.unreadCount || 0), 0);

export const selectActiveAccount = (state: MessengerState): MessengerAccount | undefined =>
  state.accounts.find((a) => a.index === state.activeAccountIndex);

export const selectAccountByIndex = (state: MessengerState, index: number): MessengerAccount | undefined =>
  state.accounts.find((a) => a.index === index);

export const selectVisibleBubbleCount = (state: MessengerState): number =>
  state.accounts.filter((a) => a.isConnected && a.isBubbleVisible).length;

export const selectAreAllBubblesVisible = (state: MessengerState): boolean => {
  const connected = state.accounts.filter((a) => a.isConnected);
  return connected.length > 0 && connected.every((a) => a.isBubbleVisible);
};

export const selectAreAnyBubblesVisible = (state: MessengerState): boolean =>
  state.accounts.some((a) => a.isConnected && a.isBubbleVisible);

export const selectIsAccountBubbleVisible = (state: MessengerState, accountIndex: number): boolean => {
  const acc = state.accounts.find((a) => a.index === accountIndex);
  return acc ? acc.isConnected && acc.isBubbleVisible : false;
};
