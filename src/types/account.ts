import { getAccountThemeColor } from "../constants/design";

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

export type ProcessLifecycleState = "cold" | "spawning" | "active" | "suspending";

export interface MessengerAccount {
  id: string; // e.g. "messenger-01", "messenger-02"
  index: number; // 1..5
  name: string; // "Messenger 01"
  avatarText: string; // "M1"
  color: string; // Account theme hex color
  isConnected: boolean;
  isAuthenticating?: boolean;
  isBubbleVisible: boolean; // Current active bubble display state
  shouldRestoreBubble?: boolean; // Auto-restore bubble when loaded
  lifecycleState?: ProcessLifecycleState; // Process lifecycle state (Cold Standby vs Active)
  unreadCount: number;
  lastSynced?: string;
  latestSenderAvatar?: string;
  hasNewMessagePulse?: boolean;
  soundEnabled?: boolean;
  hideMessageContent?: boolean;
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
    lifecycleState: "cold",
    unreadCount: 0,
    lastSynced: "Not connected",
    soundEnabled: true,
    hideMessageContent: false,
  };
}
