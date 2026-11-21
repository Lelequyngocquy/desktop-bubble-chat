/**
 * Strongly-Typed IPC Contracts & Constants
 */

// 1. IPC Command Names
export const IPC_COMMANDS = {
  // System Commands
  GET_SYSTEM_USERNAME: "get_system_username",
  TOGGLE_AUTOSTART: "toggle_autostart",
  IS_AUTOSTART_ENABLED: "is_autostart_enabled",
  OPEN_EXTERNAL_URL: "open_external_url",
  OPEN_PROFILES_FOLDER: "open_profiles_folder",
  EXIT_APP: "exit_app",
  RESTART_APP: "restart_app",
  MINIMIZE_MAIN_WINDOW: "minimize_main_window",
  TOGGLE_MAXIMIZE_MAIN_WINDOW: "toggle_maximize_main_window",
  HIDE_MAIN_WINDOW: "hide_main_window",
  SHOW_MAIN_WINDOW: "show_main_window",
  SET_ALWAYS_ON_TOP: "set_always_on_top",
  UPDATE_TRAY_MENU_STATUS: "update_tray_menu_status",
  LOG_FRONTEND: "log_frontend",
  GET_USER_SETTINGS: "get_user_settings",
  SET_APP_LANGUAGE: "set_app_language",
  SET_AUTO_TRIM_ENABLED: "set_auto_trim_enabled",
  SET_THEME_MODE: "set_theme_mode",
  SET_UPDATE_MODE: "set_update_mode",
  SET_GLOBAL_SHORTCUT_ENABLED: "set_global_shortcut_enabled",

  // Performance Commands
  GET_PERFORMANCE_SNAPSHOT: "get_performance_snapshot",
  TRIM_SYSTEM_MEMORY: "trim_system_memory",

  // Badge Commands
  SET_BADGE_COUNT: "set_badge_count",
  GET_ALL_UNREAD_COUNTS: "get_all_unread_counts",

  // Bubble Window & Positioning Commands
  TOGGLE_BUBBLE: "toggle_bubble",
  TOGGLE_ALL_BUBBLES: "toggle_all_bubbles",
  GET_ACTIVE_BUBBLE_ACCOUNTS: "get_active_bubble_accounts",
  SET_BUBBLE_CONFIG: "set_bubble_config",
  SET_BUBBLE_POSITION_MODE: "set_bubble_position_mode",

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
  REPORT_INCOMING_NOTIFICATION: "report_incoming_notification",
  TOGGLE_PIN_CHAT: "toggle_pin_chat",
  GET_RECENT_MESSAGES: "get_recent_messages",
  SET_ACCOUNT_MUTED: "set_account_muted",
  SET_CHAT_WINDOW_SIZE_CONFIG: "set_chat_window_size_config",

  // Extension Commands
  OPEN_PROFILE_EXTENSIONS_FOLDER: "open_profile_extensions_folder",
} as const;

export type IpcCommand = (typeof IPC_COMMANDS)[keyof typeof IPC_COMMANDS];

// 2. IPC Event Names
export const IPC_EVENTS = {
  BADGE_UPDATED: "badge-updated",
  MESSENGER_AUTH_STATUS: "messenger-auth-status",
  MESSENGER_AUTH_CLOSED: "messenger-auth-closed",
  BUBBLE_VISIBILITY_CHANGED: "bubble-visibility-changed",
  BUBBLE_SPAWNED: "bubble-spawned",
  BUBBLE_CONFIG_CHANGED: "bubble-config-changed",
  CHAT_VISIBILITY_CHANGED: "chat-visibility-changed",
  CHAT_PIN_CHANGED: "chat-pin-changed",
  NEW_MESSAGE_RECEIVED: "new-message-received",
  RECENT_CONVERSATIONS_SYNCED: "recent-conversations-synced",
  USER_SETTINGS_CHANGED: "user-settings-changed",
  SHOW_CLOSE_DIALOG: "show-close-dialog",
  CHAT_WINDOW_RESIZED: "chat-window-resized",
  UPDATER_STATUS: "updater-status",
} as const;

export type IpcEvent = (typeof IPC_EVENTS)[keyof typeof IPC_EVENTS];

// 3. Event Payloads
export interface UserSettingsSnapshot {
  is_dashboard_open?: boolean;
  isDashboardOpen?: boolean;
  active_bubble_accounts?: number[];
  activeBubbleAccounts?: number[];
  is_notifications_enabled?: boolean;
  isNotificationsEnabled?: boolean;
  muted_accounts?: number[];
  mutedAccounts?: number[];
  bubble_size?: "sm" | "md" | "lg";
  bubbleSize?: "sm" | "md" | "lg";
  is_always_on_top?: boolean;
  isAlwaysOnTop?: boolean;
  is_auto_start?: boolean;
  isAutoStart?: boolean;
  locale?: string | null;
  is_auto_trim_enabled?: boolean;
  isAutoTrimEnabled?: boolean;
  theme_mode?: string | null;
  themeMode?: string | null;
  bubble_position_mode?: "auto_tuck" | "free";
  bubblePositionMode?: "auto_tuck" | "free";
  update_mode?: "auto" | "manual" | null;
  updateMode?: "auto" | "manual" | null;
  authenticated_accounts?: number[];
  authenticatedAccounts?: number[];
  is_global_shortcut_enabled?: boolean;
  isGlobalShortcutEnabled?: boolean;
}

export interface ChatWindowResizedPayload {
  accountIndex: number;
  width: number;
  height: number;
}

export interface MessengerAuthStatusPayload {
  connected: boolean;
  name?: string | null;
  accountIndex?: number;
}

export interface MessengerAuthClosedPayload {
  label: string;
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
