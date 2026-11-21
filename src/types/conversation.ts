export interface Conversation {
  id: string;
  name: string;
  avatarText?: string;
  avatarUrl?: string;
  isGroup?: boolean;
  lastMessage: string;
  timestamp: string;
  timestampMs?: number;
  unreadCount: number;
  accountIndex?: number;
  threadId?: string;
  isPinned?: boolean;
}
