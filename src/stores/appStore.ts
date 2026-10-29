import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { Conversation, ConnectedAccount, NavTab } from "../types";
import { MOCK_FALLBACK_USER } from "../mocks";

export type { NavTab };

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return MOCK_FALLBACK_USER.avatarText;
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  const first = parts[0][0];
  const last = parts[parts.length - 1][0];
  return (first + last).toUpperCase();
}

interface AppState {
  // Windows System User Identity (loaded dynamically from OS)
  systemUser: string;
  systemUserInitials: string;
  fetchSystemUser: () => Promise<void>;

  // Navigation active tab
  activeTab: NavTab;
  setActiveTab: (tab: NavTab) => void;

  // Messenger connection state
  isMessengerConnected: boolean;
  setIsMessengerConnected: (connected: boolean) => void;
  connectedAccount: ConnectedAccount | null;
  setConnectedAccount: (account: ConnectedAccount | null) => void;

  // Unread badge count from Messenger
  unreadCount: number;
  setUnreadCount: (count: number) => void;

  // Window states
  isChatOpen: boolean;
  setIsChatOpen: (isOpen: boolean) => void;

  // Quick settings
  isBubbleEnabled: boolean;
  setBubbleEnabled: (enabled: boolean) => void;
  toggleBubble: () => boolean; // returns true if toggled, false if blocked

  isAlwaysOnTop: boolean;
  toggleAlwaysOnTop: () => void;

  isNotificationsEnabled: boolean;
  toggleNotifications: () => void;

  isAutoStart: boolean;
  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;

  // Pinned chat heads list (Home workspace)
  conversations: Conversation[];
  setConversations: (conversations: Conversation[]) => void;
  removeConversation: (id: string) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Windows System User Identity
  systemUser: MOCK_FALLBACK_USER.name,
  systemUserInitials: MOCK_FALLBACK_USER.avatarText,
  fetchSystemUser: async () => {
    try {
      const name = await invoke<string>("get_system_username");
      if (name && name.trim()) {
        const cleanName = name.trim();
        set({
          systemUser: cleanName,
          systemUserInitials: getInitials(cleanName),
        });
      } else {
        set({
          systemUser: MOCK_FALLBACK_USER.name,
          systemUserInitials: MOCK_FALLBACK_USER.avatarText,
        });
      }
    } catch {
      set({
        systemUser: MOCK_FALLBACK_USER.name,
        systemUserInitials: MOCK_FALLBACK_USER.avatarText,
      });
    }
  },

  activeTab: "home",
  setActiveTab: (tab) => set({ activeTab: tab }),

  // Fresh install / Zero-data default
  isMessengerConnected: false,
  setIsMessengerConnected: (connected) =>
    set((state) => ({
      isMessengerConnected: connected,
      // If disconnected, automatically disable bubble
      isBubbleEnabled: connected ? state.isBubbleEnabled : false,
      unreadCount: connected ? state.unreadCount : 0,
      connectedAccount: connected ? state.connectedAccount : null,
    })),
  connectedAccount: null,
  setConnectedAccount: (account) => set({ connectedAccount: account }),

  unreadCount: 0,
  setUnreadCount: (count) => set({ unreadCount: count }),

  isChatOpen: false,
  setIsChatOpen: (isOpen) => set({ isChatOpen: isOpen }),

  isBubbleEnabled: false,
  setBubbleEnabled: (enabled) => set({ isBubbleEnabled: enabled }),
  toggleBubble: () => {
    const { isMessengerConnected, isBubbleEnabled } = get();
    // Cannot enable bubble if Messenger is not connected
    if (!isMessengerConnected && !isBubbleEnabled) {
      return false;
    }
    set({ isBubbleEnabled: !isBubbleEnabled });
    return true;
  },

  isAlwaysOnTop: true,
  toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),

  isNotificationsEnabled: true,
  toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),

  isAutoStart: false,
  setAutoStart: (enabled) => set({ isAutoStart: enabled }),
  toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),

  // Fresh install: No mock conversations by default
  conversations: [],
  setConversations: (conversations) => set({ conversations }),
  removeConversation: (id) =>
    set((state) => ({
      conversations: state.conversations.filter((c) => c.id !== id),
    })),
}));
