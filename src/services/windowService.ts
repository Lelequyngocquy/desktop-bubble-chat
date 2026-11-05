import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { IPC_COMMANDS } from "../types/ipc";

export const windowService = {
  /**
   * Minimizes the main window to taskbar.
   */
  async minimize(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.MINIMIZE_MAIN_WINDOW);
    } catch {
      try {
        await getCurrentWindow().minimize();
      } catch {}
    }
  },

  /**
   * Toggles maximize/unmaximize on the main window.
   */
  async toggleMaximize(): Promise<boolean> {
    try {
      return await invoke<boolean>(IPC_COMMANDS.TOGGLE_MAXIMIZE_MAIN_WINDOW);
    } catch {
      try {
        const win = getCurrentWindow();
        await win.toggleMaximize();
        return await win.isMaximized();
      } catch {
        return false;
      }
    }
  },

  /**
   * Hides the main dashboard window to system tray.
   */
  async hide(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.HIDE_MAIN_WINDOW);
    } catch {
      try {
        await getCurrentWindow().hide();
      } catch {}
    }
  },

  /**
   * Shows or creates the main dashboard window.
   */
  async show(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.SHOW_MAIN_WINDOW);
    } catch {
      try {
        await getCurrentWindow().show();
      } catch {}
    }
  },

  /**
   * Exits the application completely.
   */
  async exit(): Promise<void> {
    try {
      await invoke(IPC_COMMANDS.EXIT_APP);
    } catch {
      try {
        await getCurrentWindow().destroy();
      } catch {}
    }
  },

  /**
   * Checks if the current window is currently maximized.
   */
  async isMaximized(): Promise<boolean> {
    try {
      return await getCurrentWindow().isMaximized();
    } catch {
      return false;
    }
  },

  /**
   * Subscribes to window resize events to detect maximization changes.
   */
  async onResized(callback: () => void): Promise<(() => void) | undefined> {
    try {
      return await getCurrentWindow().onResized(callback);
    } catch {
      return undefined;
    }
  },
};
