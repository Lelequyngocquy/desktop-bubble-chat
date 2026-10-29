import { Conversation } from "../types/conversation";

export const MOCK_CONVERSATIONS: Conversation[] = [
  {
    id: "conv-1",
    name: "Nguyễn Văn An",
    avatarText: "NA",
    lastMessage: "Tối nay có rảnh trao đổi công việc không bạn?",
    timestamp: "10:45",
    unreadCount: 2,
  },
  {
    id: "conv-2",
    name: "Nhóm Dự án Desktop",
    avatarText: "DA",
    isGroup: true,
    lastMessage: "Bản build mới đã test thành công trên Windows 11.",
    timestamp: "09:30",
    unreadCount: 0,
  },
  {
    id: "conv-3",
    name: "Trần Thị Mai",
    avatarText: "TM",
    lastMessage: "Đã gửi tài liệu thiết kế qua mail rồi nhé.",
    timestamp: "Hôm qua",
    unreadCount: 1,
  },
];
