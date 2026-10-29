import { invoke } from "@tauri-apps/api/core";
import { MessengerAuthStatus } from "./types";

/**
 * Service encapsulating all Tauri IPC commands for the Messenger platform.
 */
export const messengerService = {
  /**
   * Check authentication status of the Messenger webview session.
   */
  async checkAuth(): Promise<MessengerAuthStatus> {
    return await invoke<MessengerAuthStatus>("check_messenger_auth");
  },

  /**
   * Open the Messenger auth window to allow user to log in.
   */
  async openAuth(): Promise<void> {
    await invoke("open_messenger_auth");
  },

  /**
   * Show and focus the Messenger chat popup window.
   */
  async openChat(): Promise<void> {
    await invoke("open_chat");
  },

  /**
   * Toggle visibility of the Messenger chat popup window.
   */
  async toggleChat(): Promise<boolean> {
    return await invoke<boolean>("toggle_chat");
  },

  /**
   * Refresh/reload the Messenger webview and check status.
   */
  async refresh(): Promise<void> {
    await invoke("refresh_messenger");
  },

  /**
   * Disconnect Messenger (clear session cookies/cache and hide window).
   */
  async disconnect(): Promise<void> {
    await invoke("disconnect_messenger");
  },

  /**
   * Explicitly report authentication status from the webview injection script.
   */
  async reportAuthStatus(connected: boolean, name?: string): Promise<void> {
    await invoke("report_auth_status", { connected, name });
  },
};
