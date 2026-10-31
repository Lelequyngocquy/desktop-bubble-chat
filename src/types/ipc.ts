/**
 * Strongly-Typed IPC Contracts & Constants
 * Eliminates magic strings, ensures compile-time safety across Frontend-Backend boundary.
 */

// 1. IPC Command Names
export const IPC_COMMANDS = {
  // System Commands
  GET_SYSTEM_USERNAME: "get_system_username",
  TOGGLE_AUTOSTART: "toggle_autostart",
  IS_AUTOSTART_ENABLED: "is_autostart_enabled",
  OPEN_EXTERNAL_URL: "open_external_url",
  EXIT_APP: "exit_app",
  SET_ALWAYS_ON_TOP: "set_always_on_top",
  UPDATE_TRAY_MENU_STATUS: "update_tray_menu_status",
  LOG_FRONTEND: "log_frontend",

  // Badge Commands
  SET_BADGE_COUNT: "set_badge_count",
  UPDATE_UNREAD_BADGE: "update_unread_badge",

  // Bubble Window & Positioning Commands
  TOGGLE_BUBBLE: "toggle_bubble",
  TOGGLE_ALL_BUBBLES: "toggle_all_bubbles",
  SNAP_BUBBLE: "snap_bubble",
  GET_BUBBLE_POSITION: "get_bubble_position",
  GET_ALL_BUBBLE_POSITIONS: "get_all_bubble_positions",
  GET_BUBBLE_SNAP_COORDS: "get_bubble_snap_coords",
  GET_BUBBLE_DEFAULT_COORDS: "get_bubble_default_coords",
  GET_BUBBLE_HOVER_COORDS: "get_bubble_hover_coords",
  SET_BUBBLE_CONFIG: "set_bubble_config",

  // Messenger Platform Commands
  OPEN_MESSENGER_AUTH: "open_messenger_auth",
  CHECK_MESSENGER_AUTH: "check_messenger_auth",
  OPEN_CHAT: "open_chat",
  PRELOAD_CHAT: "preload_chat",
  OPEN_THREAD: "open_thread",
  TOGGLE_CHAT: "toggle_chat",
  REFRESH_MESSENGER: "refresh_messenger",
  DISCONNECT_MESSENGER: "disconnect_messenger",
  CLOSE_MESSENGER_WINDOW: "close_messenger_window",
  CLEAR_ACCOUNT_STORAGE: "clear_account_storage",
  REPORT_AUTH_STATUS: "report_auth_status",
  REPORT_NEW_MESSAGE: "report_new_message",
  SYNC_RECENT_CONVERSATIONS: "sync_recent_conversations",
} as const;

export type IpcCommand = (typeof IPC_COMMANDS)[keyof typeof IPC_COMMANDS];

// 2. IPC Event Names
export const IPC_EVENTS = {
  BADGE_UPDATED: "badge-updated",
  MESSENGER_AUTH_STATUS: "messenger-auth-status",
  MESSENGER_AUTH_CLOSED: "messenger-auth-closed",
  BUBBLE_VISIBILITY_CHANGED: "bubble-visibility-changed",
  BUBBLE_SPAWNED: "bubble-spawned",
  CHAT_VISIBILITY_CHANGED: "chat-visibility-changed",
  NEW_MESSAGE_RECEIVED: "new-message-received",
  RECENT_CONVERSATIONS_SYNCED: "recent-conversations-synced",
  BUBBLE_CONFIG_CHANGED: "bubble-config-changed",
} as const;

export type IpcEvent = (typeof IPC_EVENTS)[keyof typeof IPC_EVENTS];

// 3. Event Payloads
export interface MessengerAuthStatusPayload {
  connected: boolean;
  name?: string | null;
  accountIndex?: number;
  url?: string;
  title?: string;
}

export interface MessengerAuthClosedPayload {
  label: string;
}

export interface SnapResultPayload {
  targetX: number;
  targetY: number;
  shouldDismiss: boolean;
  isLeftEdge?: boolean;
}

export interface BubblePositionPayload {
  accountIndex: number;
  x: number;
  y: number;
  width: number;
  height: number;
  isVisible: boolean;
}

export interface BadgeUpdatedPayload {
  accountIndex: number;
  count: number;
}

export interface BubbleVisibilityPayload {
  accountIndex: number;
  visible: boolean;
  x?: number;
  y?: number;
}

export interface BubbleSpawnedPayload {
  accountIndex: number;
  x: number;
  y: number;
}

export interface ToggleAllBubblesPayload {
  visible?: boolean;
  accountIndices?: number[];
}

export interface BubbleConfigPayload {
  size?: number;
  tuckRatio?: number;
}

export interface BubbleConfigChangedPayload {
  size: number;
  bubbleSize: "sm" | "md" | "lg";
  tuckRatio: number;
}

export interface IncomingMessagePayload {
  accountIndex: number;
  senderName: string;
  body?: string;
  avatarUrl?: string;
  threadId?: string;
  timestamp: number;
}

export interface ChatVisibilityPayload {
  accountIndex: number;
  visible: boolean;
  windowLabel?: string;
}

export interface RecentConversationsSyncedPayload {
  accountIndex: number;
  conversations: IncomingMessagePayload[];
}

