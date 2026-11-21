import { create } from "zustand";
import type { TFunction } from "i18next";
import { Conversation } from "../types/conversation";
import { IncomingMessagePayload } from "../types/ipc";
import { normalizeThreadId } from "../utils/threadValidation";

if (typeof window !== "undefined" && window.localStorage) {
  try {
    window.localStorage.removeItem("desktop_bubble_chat_conversations");
  } catch (err) {
    console.warn("[conversationStore] Could not prune legacy storage key:", err);
  }
}

export function formatMessageTime(
  timestamp?: number,
  t?: TFunction<"dashboard">,
): string {
  if (!timestamp) return t ? t("time.just_now") : "Just now";
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return t ? t("time.just_now") : "Just now";
  const now = new Date();
  const diffMinutes = Math.floor((now.getTime() - date.getTime()) / 60000);
  if (diffMinutes < 1) return t ? t("time.just_now") : "Just now";
  if (diffMinutes < 60) return t ? t("time.minutes_ago", { count: diffMinutes }) : `${diffMinutes}m ago`;
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

function buildConversationFallbackId(accountIndex: number, displayName: string): string {
  return `sender-${accountIndex}-${displayName.trim().toLowerCase()}`;
}

function findMatchingConversationIndex(
  list: Conversation[],
  accountIndex: number,
  displayName: string,
  validThreadId?: string,
): number {
  if (validThreadId) {
    const byThread = list.findIndex((c) => c.threadId === validThreadId || c.id === validThreadId);
    if (byThread >= 0) return byThread;
  }
  const normalizedName = displayName.trim().toLowerCase();
  return list.findIndex(
    (c) => (c.accountIndex ?? 1) === accountIndex && c.name.trim().toLowerCase() === normalizedName,
  );
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
      const accountIdx = msg.accountIndex || 1;
      const displayName = formatSenderName(msg.senderName, accountIdx);
      const validThreadId = normalizeThreadId(msg.threadId);
      const existingIndex = findMatchingConversationIndex(state.conversations, accountIdx, displayName, validThreadId);
      const existing = existingIndex >= 0 ? state.conversations[existingIndex] : null;
      const existingThreadId = normalizeThreadId(existing?.threadId);
      const resolvedThreadId = validThreadId || existingThreadId;
      const timestampMs = msg.timestamp || Date.now();
      const updatedItem: Conversation = {
        id: existing ? existing.id : resolvedThreadId || buildConversationFallbackId(accountIdx, displayName),
        name: displayName,
        avatarUrl: "",
        avatarText: "",
        lastMessage: msg.body && msg.body.trim().length > 0 ? msg.body : "New message",
        timestamp: formatMessageTime(timestampMs),
        timestampMs,
        unreadCount: (existing?.unreadCount || 0) + 1,
        accountIndex: accountIdx,
        threadId: resolvedThreadId,
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
      const current = [...state.conversations];
      const reversed = [...list].reverse();
      for (const msg of reversed) {
        const accountIdx = msg.accountIndex || 1;
        const displayName = formatSenderName(msg.senderName, accountIdx);
        const validThreadId = normalizeThreadId(msg.threadId);
        const existingIndex = findMatchingConversationIndex(current, accountIdx, displayName, validThreadId);
        const existing = existingIndex >= 0 ? current[existingIndex] : null;
        const existingThreadId = normalizeThreadId(existing?.threadId);
        const resolvedThreadId = validThreadId || existingThreadId;
        const timestampMs = msg.timestamp || Date.now();
        const updatedItem: Conversation = {
          id: existing ? existing.id : resolvedThreadId || buildConversationFallbackId(accountIdx, displayName),
          name: displayName,
          avatarUrl: "",
          avatarText: "",
          lastMessage: msg.body && msg.body.trim().length > 0 ? msg.body : "New message",
          timestamp: formatMessageTime(timestampMs),
          timestampMs,
          unreadCount: existing?.unreadCount || 0,
          accountIndex: accountIdx,
          threadId: resolvedThreadId,
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
      const updated = state.conversations.map((c) => (c.id === id ? { ...c, isPinned: !c.isPinned } : c));
      return { conversations: updated.sort((a, b) => (b.isPinned ? 1 : 0) - (a.isPinned ? 1 : 0)) };
    }),
  markConversationRead: (id) =>
    set((state) => ({
      conversations: state.conversations.map((c) => (c.id === id ? { ...c, unreadCount: 0 } : c)),
    })),
  syncRecentConversations: (accountIndex, list) =>
    set((state) => {
      const accountIdx = accountIndex || 1;
      const updated = [...state.conversations];
      for (const item of list) {
        const displayName = formatSenderName(item.senderName, accountIdx);
        const validThreadId = normalizeThreadId(item.threadId);
        const idx = findMatchingConversationIndex(updated, accountIdx, displayName, validThreadId);
        if (idx >= 0) {
          const existingThreadId = normalizeThreadId(updated[idx].threadId);
          updated[idx] = {
            ...updated[idx],
            threadId: validThreadId || existingThreadId,
            avatarUrl: item.avatarUrl || updated[idx].avatarUrl,
            accountIndex: accountIdx,
            timestampMs: item.timestamp || updated[idx].timestampMs,
          };
        } else {
          const timestampMs = item.timestamp || Date.now();
          updated.push({
            id: validThreadId || buildConversationFallbackId(accountIdx, displayName),
            name: displayName,
            avatarUrl: item.avatarUrl,
            avatarText: displayName.slice(0, 2).toUpperCase(),
            lastMessage: item.body || "Sent a message",
            timestamp: formatMessageTime(timestampMs),
            timestampMs,
            unreadCount: 0,
            accountIndex: accountIdx,
            threadId: validThreadId,
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
