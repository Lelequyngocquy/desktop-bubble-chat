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
