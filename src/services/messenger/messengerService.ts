import { invoke } from "@tauri-apps/api/core";
import { MessengerAuthStatus } from "./types";
import { IPC_COMMANDS } from "../../types/ipc";

/**
 * Service encapsulating all Tauri IPC commands for the Messenger platform.
 * Supports multi-account operations with isolated WebView2 sessions.
 */
export const messengerService = {
  /**
   * Check authentication status of a specific Messenger account session.
   */
  async checkAuth(accountIndex: number = 1): Promise<MessengerAuthStatus> {
    return await invoke<MessengerAuthStatus>(IPC_COMMANDS.CHECK_MESSENGER_AUTH, {
      accountIndex,
    });
  },

  /**
   * Open the Messenger auth window for a specific account.
   */
  async openAuth(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.OPEN_MESSENGER_AUTH, { accountIndex });
  },

  /**
   * Show and focus the Messenger chat popup window for a specific account.
   */
  async openChat(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.OPEN_CHAT, { accountIndex });
  },

  /**
   * Preload the Messenger chat webview in background for instant hover response.
   */
  async preloadChat(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.PRELOAD_CHAT, { accountIndex }).catch(() => {});
  },

  /**
   * Open a specific conversation thread and dock chat window next to Bubble.
   */
  async openThread(threadId: string, accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.OPEN_THREAD, { threadId, accountIndex });
  },

  /**
   * Toggle visibility of the Messenger chat popup window for a specific account.
   */
  async toggleChat(accountIndex: number = 1): Promise<boolean> {
    return await invoke<boolean>(IPC_COMMANDS.TOGGLE_CHAT, { accountIndex });
  },

  /**
   * Refresh/reload the Messenger webview session.
   */
  async refresh(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.REFRESH_MESSENGER, { accountIndex });
  },

  /**
   * Disconnect Messenger account (clear session / navigate to login and hide window).
   */
  async disconnect(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.DISCONNECT_MESSENGER, { accountIndex });
  },

  /**
   * Close and destroy the webview window for a specific account.
   */
  async closeWindow(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.CLOSE_MESSENGER_WINDOW, { accountIndex });
  },

  /**
   * Completely clear cache, cookies, and storage directory for a specific account.
   */
  async clearStorage(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.CLEAR_ACCOUNT_STORAGE, { accountIndex });
  },

  /**
   * Explicitly report authentication status from the webview injection script.
   */
  async reportAuthStatus(
    connected: boolean,
    name?: string,
    accountIndex: number = 1
  ): Promise<void> {
    await invoke(IPC_COMMANDS.REPORT_AUTH_STATUS, { connected, name, accountIndex });
  },

  /**
   * Toggle or show/hide the bubble window for a specific account.
   */
  async toggleBubble(visible: boolean, accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.TOGGLE_BUBBLE, { visible, accountIndex });
  },

  /**
   * Toggle or show/hide all bubble windows for specified account indices (or all 1..5).
   * Spawns bubbles sequentially with staggered commits so they never overlap.
   */
  async toggleAllBubbles(visible?: boolean, accountIndices: number[] = [1, 2, 3, 4, 5]): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.TOGGLE_ALL_BUBBLES, { visible, accountIndices });
    } catch {
      // Fallback: Tuần tự hoá tuyệt đối nếu IPC gặp sự cố
      for (const idx of accountIndices) {
        await invoke(IPC_COMMANDS.TOGGLE_BUBBLE, { visible, accountIndex: idx });
        if (visible) {
          await new Promise((r) => setTimeout(r, 70));
        }
      }
    }
  },

  /**
   * Cấu hình kích thước và tỉ lệ lấn mép của Bong bóng nổi.
   */
  async setBubbleConfig(size?: number, tuckRatio?: number): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SET_BUBBLE_CONFIG, { size, tuckRatio });
    } catch (e) {
      console.warn("[MessengerService] Failed to set bubble config:", e);
    }
  },
};
