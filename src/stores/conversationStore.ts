import { create } from "zustand";
import { Conversation } from "../types/conversation";
import { IncomingMessagePayload } from "../types/ipc";

if (typeof window !== "undefined" && window.localStorage) {
  try {
    window.localStorage.removeItem("desktop_bubble_chat_conversations");
  } catch {}
}

export function formatMessageTime(timestamp?: number): string {
  if (!timestamp) return "Just now";
  const date = new Date(timestamp);
  const now = new Date();
  const diffMinutes = Math.floor((now.getTime() - date.getTime()) / 60000);
  if (diffMinutes < 1) return "Just now";
  if (diffMinutes < 60) return `${diffMinutes}m ago`;
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${hours}:${minutes}`;
}

export function formatSenderName(name: string, accountIndex?: number): string {
  const trimmed = (name || "").trim();
  const match = trimmed.match(/^M0?(\d+)$/i);
  if (match) {
    const idx = parseInt(match[1], 10);
    return `New message for Messenger ${idx.toString().padStart(2, "0")}`;
  }
  if (!trimmed) {
    const idx = accountIndex || 1;
    return `New message for Messenger ${idx.toString().padStart(2, "0")}`;
  }
  return trimmed;
}

interface ConversationState {
  conversations: Conversation[];
  setConversations: (conversations: Conversation[]) => void;
  removeConversation: (id: string) => void;
  clearAccountConversations: (accountIndex: number) => void;
  upsertConversationFromMessage: (msg: IncomingMessagePayload) => void;
  togglePinConversation: (id: string) => void;
  markConversationRead: (id: string) => void;
  syncRecentConversations: (accountIndex: number, list: IncomingMessagePayload[]) => void;
  syncRecentIncomingMessages: (list: IncomingMessagePayload[]) => void;
}

export const useConversationStore = create<ConversationState>()((set) => ({
  conversations: [],
  setConversations: (conversations) =>
    set({
      conversations: conversations.map((c) => ({
        ...c,
        name: formatSenderName(c.name, c.accountIndex),
        avatarUrl: "",
        avatarText: "",
      })),
    }),
  removeConversation: (id) =>
    set((state) => ({
      conversations: state.conversations.filter((c) => c.id !== id),
    })),
  clearAccountConversations: (accountIndex) =>
    set((state) => ({
      conversations: state.conversations.filter((c) => (c.accountIndex || 1) !== accountIndex),
    })),
  upsertConversationFromMessage: (msg) =>
    set((state) => {
      const displayName = formatSenderName(msg.senderName, msg.accountIndex);
      const id = msg.threadId || (msg.accountIndex ? `account-msg-${msg.accountIndex}` : `sender-${displayName.toLowerCase()}`);
      const existingIndex = state.conversations.findIndex(
        (c) =>
          c.id === id ||
          c.name.toLowerCase() === displayName.toLowerCase() ||
          (msg.accountIndex && c.accountIndex === msg.accountIndex)
      );
      const existing = existingIndex >= 0 ? state.conversations[existingIndex] : null;
      const updatedItem: Conversation = {
        id: existing ? existing.id : id,
        name: displayName,
        avatarUrl: "",
        avatarText: "",
        lastMessage: msg.body && msg.body.trim().length > 0 ? msg.body : "New message",
        timestamp: formatMessageTime(msg.timestamp),
        unreadCount: (existing?.unreadCount || 0) + 1,
        accountIndex: msg.accountIndex,
        threadId: msg.threadId || existing?.threadId,
        isPinned: existing?.isPinned ?? false,
      };
      const filtered = state.conversations.filter((_, idx) => idx !== existingIndex);
      const pinned = filtered.filter((c) => c.isPinned);
      const unpinned = filtered.filter((c) => !c.isPinned);
      if (updatedItem.isPinned) {
        return { conversations: [updatedItem, ...pinned, ...unpinned].slice(0, 25) };
      }
      return { conversations: [...pinned, updatedItem, ...unpinned].slice(0, 25) };
    }),
  syncRecentIncomingMessages: (list) =>
    set((state) => {
      if (!list || list.length === 0) return state;
      let current = [...state.conversations];
      const reversed = [...list].reverse();
      for (const msg of reversed) {
        const displayName = formatSenderName(msg.senderName, msg.accountIndex);
        const id = msg.threadId || (msg.accountIndex ? `account-msg-${msg.accountIndex}` : `sender-${displayName.toLowerCase()}`);
        const existingIndex = current.findIndex(
          (c) =>
            c.id === id ||
            c.name.toLowerCase() === displayName.toLowerCase() ||
            (msg.accountIndex && c.accountIndex === msg.accountIndex)
        );
        const existing = existingIndex >= 0 ? current[existingIndex] : null;
        const updatedItem: Conversation = {
          id: existing ? existing.id : id,
          name: displayName,
          avatarUrl: "",
          avatarText: "",
          lastMessage: msg.body && msg.body.trim().length > 0 ? msg.body : "New message",
          timestamp: formatMessageTime(msg.timestamp),
          unreadCount: existing?.unreadCount || 0,
          accountIndex: msg.accountIndex,
          threadId: msg.threadId || existing?.threadId,
          isPinned: existing?.isPinned ?? false,
        };
        if (existingIndex >= 0) {
          current[existingIndex] = updatedItem;
        } else {
          current.unshift(updatedItem);
        }
      }
      const pinned = current.filter((c) => c.isPinned);
      const unpinned = current.filter((c) => !c.isPinned);
      return { conversations: [...pinned, ...unpinned].slice(0, 25) };
    }),
  togglePinConversation: (id) =>
    set((state) => {
      const item = state.conversations.find((c) => c.id === id);
      if (!item) return state;
      const currentlyPinnedCount = state.conversations.filter((c) => c.isPinned).length;
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
            lastMessage: item.body || "Sent a message",
            timestamp: "Today",
            unreadCount: 0,
            accountIndex: accountIndex,
            threadId: item.threadId,
            isPinned: false,
          });
        }
      }
      return {
        conversations: updated
          .sort((a, b) => (b.isPinned ? 1 : 0) - (a.isPinned ? 1 : 0))
          .slice(0, 25),
      };
    }),
}));
