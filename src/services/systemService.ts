import { invoke } from "@tauri-apps/api/core";
import { IPC_COMMANDS } from "../types/ipc";
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
   * Gets persisted user settings snapshot from Backend cache.
   */
  async getUserSettings(): Promise<{
    is_dashboard_open: boolean;
    active_bubble_accounts: number[];
    is_notifications_enabled: boolean;
    muted_accounts: number[];
    bubble_size: "sm" | "md" | "lg";
    is_always_on_top: boolean;
    is_auto_start: boolean;
    locale?: string;
  } | null> {
    try {
      return await invoke(IPC_COMMANDS.GET_USER_SETTINGS);
    } catch (err) {
      console.warn("[SystemService] Failed to get user settings:", err);
      return null;
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
