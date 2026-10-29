import { create } from "zustand";

export interface Conversation {
  id: string;
  name: string;
  avatarText?: string;
  avatarUrl?: string;
  isGroup?: boolean;
  lastMessage: string;
  timestamp: string;
  unreadCount: number;
}

interface AppState {
  // Unread badge count from Messenger
  unreadCount: number;
  setUnreadCount: (count: number) => void;

  // Window states
  isChatOpen: boolean;
  setIsChatOpen: (isOpen: boolean) => void;

  // Quick settings
  isBubbleEnabled: boolean;
  setBubbleEnabled: (enabled: boolean) => void;
  toggleBubble: () => void;

  isAlwaysOnTop: boolean;
  toggleAlwaysOnTop: () => void;

  isNotificationsEnabled: boolean;
  toggleNotifications: () => void;

  isAutoStart: boolean;
  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;

  // Pinned chat heads list (Home workspace)
  conversations: Conversation[];
  removeConversation: (id: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  unreadCount: 5,
  setUnreadCount: (count) => set({ unreadCount: count }),

  isChatOpen: false,
  setIsChatOpen: (isOpen) => set({ isChatOpen: isOpen }),

  isBubbleEnabled: true,
  setBubbleEnabled: (enabled) => set({ isBubbleEnabled: enabled }),
  toggleBubble: () => set((state) => ({ isBubbleEnabled: !state.isBubbleEnabled })),

  isAlwaysOnTop: true,
  toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),

  isNotificationsEnabled: true,
  toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),

  isAutoStart: false,
  setAutoStart: (enabled) => set({ isAutoStart: enabled }),
  toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),

  conversations: [
    {
      id: "1",
      name: "Linh Nguyễn",
      avatarText: "LN",
      lastMessage: "Chiều nay mình gặp lúc 3 giờ nhé!",
      timestamp: "2 phút",
      unreadCount: 2,
    },
    {
      id: "2",
      name: "Team Design",
      isGroup: true,
      lastMessage: "Huy: Mình vừa cập nhật bản thiết kế.",
      timestamp: "5 phút",
      unreadCount: 3,
    },
    {
      id: "3",
      name: "Hoàng Nam",
      avatarText: "HN",
      lastMessage: "Bạn: Cảm ơn Nam, hẹn gặp lại 👋",
      timestamp: "12 phút",
      unreadCount: 0,
    },
  ],
  removeConversation: (id) =>
    set((state) => ({
      conversations: state.conversations.filter((c) => c.id !== id),
    })),
}));
