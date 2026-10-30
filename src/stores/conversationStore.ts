import { create } from "zustand";
import { persist } from "zustand/middleware";
import { Conversation } from "../types/conversation";
import { IncomingMessagePayload } from "../types/ipc";

interface ConversationState {
  conversations: Conversation[];
  setConversations: (conversations: Conversation[]) => void;
  removeConversation: (id: string) => void;
  upsertConversationFromMessage: (msg: IncomingMessagePayload) => void;
  togglePinConversation: (id: string) => void;
  markConversationRead: (id: string) => void;
  syncRecentConversations: (accountIndex: number, list: IncomingMessagePayload[]) => void;
}

export const useConversationStore = create<ConversationState>()(
  persist(
    (set) => ({
      conversations: [],
      setConversations: (conversations) => set({ conversations }),
      removeConversation: (id) =>
        set((state) => ({
          conversations: state.conversations.filter((c) => c.id !== id),
        })),

      upsertConversationFromMessage: (msg) =>
        set((state) => {
          const id = msg.threadId || `sender-${msg.senderName.trim().toLowerCase()}`;
          const existingIndex = state.conversations.findIndex(
            (c) => c.id === id || c.name.toLowerCase() === msg.senderName.toLowerCase()
          );

          const existing = existingIndex >= 0 ? state.conversations[existingIndex] : null;

          const updatedItem: Conversation = {
            id: existing ? existing.id : id,
            name: msg.senderName,
            avatarUrl: msg.avatarUrl || existing?.avatarUrl,
            avatarText: msg.senderName.slice(0, 2).toUpperCase(),
            lastMessage:
              msg.body && msg.body.trim().length > 0 ? msg.body : "Đã gửi một tin nhắn",
            timestamp: "Vừa xong",
            unreadCount: (existing?.unreadCount || 0) + 1,
            accountIndex: msg.accountIndex,
            threadId: msg.threadId || existing?.threadId,
            isPinned: existing?.isPinned ?? false,
          };

          const filtered = state.conversations.filter((_, idx) => idx !== existingIndex);
          const pinned = filtered.filter((c) => c.isPinned);
          const unpinned = filtered.filter((c) => !c.isPinned);

          if (updatedItem.isPinned) {
            return { conversations: [updatedItem, ...pinned, ...unpinned].slice(0, 20) };
          }
          return { conversations: [...pinned, updatedItem, ...unpinned].slice(0, 20) };
        }),

      togglePinConversation: (id) =>
        set((state) => {
          const item = state.conversations.find((c) => c.id === id);
          if (!item) return state;

          const currentlyPinnedCount = state.conversations.filter((c) => c.isPinned).length;
          // Maximum 6 pinned conversations as per specification
          if (!item.isPinned && currentlyPinnedCount >= 6) {
            return state;
          }

          const updated = state.conversations.map((c) =>
            c.id === id ? { ...c, isPinned: !c.isPinned } : c
          );
          return {
            conversations: updated.sort((a, b) => (b.isPinned ? 1 : 0) - (a.isPinned ? 1 : 0)),
          };
        }),

      markConversationRead: (id) =>
        set((state) => ({
          conversations: state.conversations.map((c) =>
            c.id === id ? { ...c, unreadCount: 0 } : c
          ),
        })),

      syncRecentConversations: (accountIndex, list) =>
        set((state) => {
          let updated = [...state.conversations];
          for (const item of list) {
            const id = item.threadId || `sender-${item.senderName.trim().toLowerCase()}`;
            const idx = updated.findIndex(
              (c) => c.id === id || c.name.toLowerCase() === item.senderName.toLowerCase()
            );

            if (idx >= 0) {
              // Update threadId and avatarUrl if missing, keep existing unread and pinned
              updated[idx] = {
                ...updated[idx],
                threadId: item.threadId || updated[idx].threadId,
                avatarUrl: item.avatarUrl || updated[idx].avatarUrl,
                accountIndex: accountIndex,
              };
            } else {
              updated.push({
                id: id,
                name: item.senderName,
                avatarUrl: item.avatarUrl,
                avatarText: item.senderName.slice(0, 2).toUpperCase(),
                lastMessage: item.body || "Đã gửi một tin nhắn",
                timestamp: "Hôm nay",
                unreadCount: 0,
                accountIndex: accountIndex,
                threadId: item.threadId,
                isPinned: false,
              });
            }
          }

          // Sort: pinned first
          return {
            conversations: updated
              .sort((a, b) => (b.isPinned ? 1 : 0) - (a.isPinned ? 1 : 0))
              .slice(0, 25),
          };
        }),
    }),
    {
      name: "desktop_bubble_chat_conversations",
    }
  )
);
