import { invoke } from "@tauri-apps/api/core";
import { IPC_COMMANDS, type UserSettingsSnapshot } from "../types/ipc";
import { SystemPerformanceSnapshot } from "../types/performance";

export const systemService = {
  /**
   * Retrieves the current Windows system username.
   */
  async getSystemUsername(): Promise<string> {
    try {
      const name = await invoke<string>(IPC_COMMANDS.GET_SYSTEM_USERNAME);
      return name?.trim() || "";
    } catch (err) {
      console.warn("[SystemService] Failed to query system username:", err);
      return "";
    }
  },

  /**
   * Checks if autostart with Windows is enabled in registry.
   */
  async isAutostartEnabled(): Promise<boolean> {
    try {
      return await invoke<boolean>(IPC_COMMANDS.IS_AUTOSTART_ENABLED);
    } catch (err) {
      console.warn("[SystemService] Failed to query autostart status:", err);
      return false;
    }
  },

  /**
   * Toggles autostart with Windows.
   */
  async toggleAutostart(enabled: boolean): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.TOGGLE_AUTOSTART, { enabled });
    } catch (err) {
      console.warn("[SystemService] Failed to toggle autostart:", err);
    }
  },

  /**
   * Sets always on top for application windows.
   */
  async setAlwaysOnTop(enabled: boolean): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_ALWAYS_ON_TOP, { enabled });
    } catch (err) {
      console.warn("[SystemService] Failed to set always on top:", err);
    }
  },

  /**
   * Updates or sets the application taskbar badge count.
   */
  async setBadgeCount(count: number): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_BADGE_COUNT, { count });
    } catch (err) {
      console.warn("[SystemService] Failed to set badge count:", err);
    }
  },

  /**
   * Opens an external URL in the default browser.
   */
  async openExternalUrl(url: string): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.OPEN_EXTERNAL_URL, { url });
    } catch (err) {
      console.warn("[SystemService] Failed to open external URL:", err);
    }
  },

  /**
   * Opens the local user data profiles directory in File Explorer.
   */
  async openProfilesFolder(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.OPEN_PROFILES_FOLDER);
    } catch (err) {
      console.warn("[SystemService] Failed to open profiles folder:", err);
    }
  },

  /**
   * Updates the System Tray context menu items according to connected account state.
   */
  async updateTrayMenu(hasConnectedAccounts: boolean): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.UPDATE_TRAY_MENU_STATUS, { hasConnectedAccounts });
    } catch (err) {
      console.warn("[SystemService] Failed to update tray menu:", err);
    }
  },

  /**
   * Sets bubble physics and dimension configuration (size in px, tuck ratio).
   */
  async setBubbleConfig(size?: number, tuckRatio?: number): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_BUBBLE_CONFIG, { size, tuckRatio });
    } catch (err) {
      console.warn("[SystemService] Failed to set bubble config:", err);
    }
  },

  /**
   * Sets bubble position mode: "auto_tuck" (default) or "free".
   */
  async setBubblePositionMode(mode: "auto_tuck" | "free"): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_BUBBLE_POSITION_MODE, { mode });
    } catch (err) {
      console.warn("[SystemService] Failed to set bubble position mode:", err);
    }
  },

  /**
   * Gets persisted user settings snapshot from Backend cache.
   */
  async getUserSettings(): Promise<UserSettingsSnapshot | null> {
    try {
      return await invoke(IPC_COMMANDS.GET_USER_SETTINGS);
    } catch (err) {
      console.warn("[SystemService] Failed to get user settings:", err);
      return null;
    }
  },

  /**
   * Sets update mode (auto or manual) in user configuration cache.
   */
  async setUpdateMode(mode: "auto" | "manual"): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_UPDATE_MODE, { mode });
    } catch (err) {
      console.warn("[SystemService] Failed to set update mode:", err);
    }
  },

  /**
   * Enables or disables global keyboard shortcuts in backend and cache.
   */
  async setGlobalShortcutEnabled(enabled: boolean): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_GLOBAL_SHORTCUT_ENABLED, { enabled });
    } catch (err) {
      console.warn("[SystemService] Failed to set global shortcut enabled:", err);
    }
  },

  /**
   * Updates application theme mode in persistent user cache.
   */
  async setThemeMode(themeMode: string): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_THEME_MODE, { themeMode });
    } catch (err) {
      console.warn("[SystemService] Failed to set theme mode:", err);
    }
  },

  /**
   * Updates application display language in persistent user cache.
   */
  async setAppLanguage(language: string): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_APP_LANGUAGE, { language });
    } catch (err) {
      console.warn("[SystemService] Failed to set app language:", err);
    }
  },

  /**
   * Opens the unpacked browser extensions folder for a specific Messenger account profile.
   */
  async openProfileExtensionsFolder(accountIndex: number): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.OPEN_PROFILE_EXTENSIONS_FOLDER, { accountIndex });
    } catch (err) {
      console.warn(`[SystemService] Failed to open extensions folder for account ${accountIndex}:`, err);
    }
  },

  /**
   * Updates auto-trim enabled configuration in persistent user cache.
   */
  async setAutoTrimEnabled(enabled: boolean): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_AUTO_TRIM_ENABLED, { enabled });
    } catch (err) {
      console.warn("[SystemService] Failed to set auto trim enabled:", err);
    }
  },

  /**
   * Completely terminates the desktop application.
   */
  async exitApp(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.EXIT_APP);
    } catch (err) {
      console.warn("[SystemService] Failed to exit application:", err);
    }
  },

  /**
   * Restarts the desktop application.
   */
  async restartApp(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.RESTART_APP);
    } catch (err) {
      console.warn("[SystemService] Failed to restart application:", err);
    }
  },

  /**
   * Retrieves real-time system performance snapshot.
   */
  async getPerformanceSnapshot(): Promise<SystemPerformanceSnapshot | null> {
    try {
      return await invoke<SystemPerformanceSnapshot>(IPC_COMMANDS.GET_PERFORMANCE_SNAPSHOT);
    } catch (err) {
      console.warn("[SystemService] Failed to get performance snapshot:", err);
      return null;
    }
  },

  /**
   * Trims working set memory of all application processes.
   */
  async trimSystemMemory(): Promise<number> {
    try {
      return await invoke<number>(IPC_COMMANDS.TRIM_SYSTEM_MEMORY);
    } catch (err) {
      console.warn("[SystemService] Failed to trim system memory:", err);
      return 0;
    }
  },
};
