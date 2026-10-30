import { invoke } from "@tauri-apps/api/core";
import { IPC_COMMANDS } from "../types/ipc";

/**
 * Real-time Logger Bridge forwarding frontend events directly to the Rust Backend terminal.
 * Solves the problem of invisible frontend console logs in Desktop Tauri apps.
 */
export const feLog = {
  info: (msg: string, ...args: unknown[]) => {
    const formatted = args.length > 0 ? `${msg} ${JSON.stringify(args)}` : msg;
    console.log(`[FE] ${formatted}`);
    invoke(IPC_COMMANDS.LOG_FRONTEND, { level: "info", message: formatted }).catch(() => {});
  },
  warn: (msg: string, ...args: unknown[]) => {
    const formatted = args.length > 0 ? `${msg} ${JSON.stringify(args)}` : msg;
    console.warn(`[FE] ${formatted}`);
    invoke(IPC_COMMANDS.LOG_FRONTEND, { level: "warn", message: formatted }).catch(() => {});
  },
  error: (msg: string, ...args: unknown[]) => {
    const formatted = args.length > 0 ? `${msg} ${JSON.stringify(args)}` : msg;
    console.error(`[FE] ${formatted}`);
    invoke(IPC_COMMANDS.LOG_FRONTEND, { level: "error", message: formatted }).catch(() => {});
  },
};
