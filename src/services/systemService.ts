import { invoke } from "@tauri-apps/api/core";
import { IPC_COMMANDS } from "../types/ipc";

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
   * Completely terminates the desktop application.
   */
  async exitApp(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.EXIT_APP);
    } catch (err) {
      console.warn("[SystemService] Failed to exit application:", err);
    }
  },
};
