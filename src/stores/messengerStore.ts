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
        break;
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
    // If updating total count for single/active account
    const activeIndex = get().activeAccountIndex;
    const current = get().accounts;
    const updated = current.map((a) => (a.index === activeIndex ? { ...a, unreadCount: count } : a));
    accountStorageService.save(updated);
    set({ accounts: updated });
  },
}));

// --- Pure Selectors (Computed from SSOT, guaranteed consistency) ---
export const selectIsMessengerConnected = (state: MessengerState): boolean =>
  state.accounts.some((a) => a.isConnected);

export const selectConnectedAccounts = (state: MessengerState): MessengerAccount[] =>
  state.accounts.filter((a) => a.isConnected);

export const selectPrimaryAccount = (state: MessengerState): MessengerAccount | null =>
  state.accounts.find((a) => a.isConnected) || null;

export const selectConnectedAccountSummary = (state: MessengerState): ConnectedAccount | null => {
  const primary = selectPrimaryAccount(state);
  if (!primary) return null;
  return {
    name: primary.name,
    avatarText: primary.avatarText,
    lastSynced: primary.lastSynced,
    color: primary.color,
  };
};

export const selectTotalUnreadCount = (state: MessengerState): number =>
  state.accounts.reduce((sum, a) => sum + (a.unreadCount || 0), 0);

export const selectActiveAccount = (state: MessengerState): MessengerAccount | undefined =>
  state.accounts.find((a) => a.index === state.activeAccountIndex);

export const selectAccountByIndex = (state: MessengerState, index: number): MessengerAccount | undefined =>
  state.accounts.find((a) => a.index === index);
