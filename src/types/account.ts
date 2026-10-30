import { getAccountThemeColor } from "../constants/accountColors";

export interface ConnectedAccount {
  name: string;
  avatarText: string;
  email?: string;
  lastSynced?: string;
  color?: string;
}

export interface SystemUserProfile {
  name: string;
  avatarText: string;
}

export const MAX_MESSENGER_ACCOUNTS = 5;

export interface MessengerAccount {
  id: string; // e.g. "messenger-01", "messenger-02"
  index: number; // 1..5
  name: string; // "Messenger 01"
  avatarText: string; // "M1"
  color: string; // Mã màu hex đại diện riêng của tài khoản
  isConnected: boolean;
  isAuthenticating?: boolean;
  isBubbleVisible: boolean; // SSOT trạng thái hiển thị bong bóng nổi của riêng tài khoản này
  unreadCount: number;
  lastSynced?: string;
  latestSenderAvatar?: string;
  hasNewMessagePulse?: boolean;
}

export function formatMessengerName(index: number): string {
  return `Messenger ${index.toString().padStart(2, "0")}`;
}

export function formatMessengerAvatarText(index: number): string {
  return `M${index}`;
}

export function createMessengerAccount(index: number): MessengerAccount {
  return {
    id: `messenger-${index.toString().padStart(2, "0")}`,
    index,
    name: formatMessengerName(index),
    avatarText: formatMessengerAvatarText(index),
    color: getAccountThemeColor(index).hex,
    isConnected: false,
    isAuthenticating: false,
    isBubbleVisible: false,
    unreadCount: 0,
    lastSynced: "Chưa kết nối",
  };
}
