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
  LOG_FRONTEND: "log_frontend",

  // Badge Commands
  SET_BADGE_COUNT: "set_badge_count",
  UPDATE_UNREAD_BADGE: "update_unread_badge",

  // Bubble Window & Positioning Commands
  TOGGLE_BUBBLE: "toggle_bubble",
  SNAP_BUBBLE: "snap_bubble",
  GET_BUBBLE_POSITION: "get_bubble_position",
  GET_ALL_BUBBLE_POSITIONS: "get_all_bubble_positions",
  GET_BUBBLE_SNAP_COORDS: "get_bubble_snap_coords",
  GET_BUBBLE_DEFAULT_COORDS: "get_bubble_default_coords",
  GET_BUBBLE_HOVER_COORDS: "get_bubble_hover_coords",

  // Messenger Platform Commands
  OPEN_MESSENGER_AUTH: "open_messenger_auth",
  CHECK_MESSENGER_AUTH: "check_messenger_auth",
  OPEN_CHAT: "open_chat",
  TOGGLE_CHAT: "toggle_chat",
  REFRESH_MESSENGER: "refresh_messenger",
  DISCONNECT_MESSENGER: "disconnect_messenger",
  CLOSE_MESSENGER_WINDOW: "close_messenger_window",
  CLEAR_ACCOUNT_STORAGE: "clear_account_storage",
  REPORT_AUTH_STATUS: "report_auth_status",
} as const;

export type IpcCommand = (typeof IPC_COMMANDS)[keyof typeof IPC_COMMANDS];

// 2. IPC Event Names
export const IPC_EVENTS = {
  BADGE_UPDATED: "badge-updated",
  MESSENGER_AUTH_STATUS: "messenger-auth-status",
  MESSENGER_AUTH_CLOSED: "messenger-auth-closed",
  BUBBLE_VISIBILITY_CHANGED: "bubble-visibility-changed",
  CHAT_VISIBILITY_CHANGED: "chat-visibility-changed",
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
}

export interface ChatVisibilityPayload {
  accountIndex: number;
  visible: boolean;
  windowLabel?: string;
}

