import { Conversation } from "../types/conversation";

export const MOCK_CONVERSATIONS: Conversation[] = [
  {
    id: "conv-1",
    name: "Alex Johnson",
    avatarText: "AJ",
    lastMessage: "Are you free to discuss the project tonight?",
    timestamp: "10:45",
    unreadCount: 2,
  },
  {
    id: "conv-2",
    name: "Desktop Project Team",
    avatarText: "DP",
    isGroup: true,
    lastMessage: "New build successfully tested on Windows 11.",
    timestamp: "09:30",
    unreadCount: 0,
  },
  {
    id: "conv-3",
    name: "Sarah Miller",
    avatarText: "SM",
    lastMessage: "Sent the design documentation via email.",
    timestamp: "Yesterday",
    unreadCount: 1,
  },
];
