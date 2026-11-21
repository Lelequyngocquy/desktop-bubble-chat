import { invoke } from "@tauri-apps/api/core";
import { IPC_COMMANDS } from "../types/ipc";

function serializeArg(arg: unknown): string {
  if (typeof arg === "string") return arg;
  if (arg instanceof Error) {
    return arg.stack || `${arg.name}: ${arg.message}`;
  }
  const seen = new WeakSet<object>();
  try {
    return JSON.stringify(arg, (_key, val) => {
      if (val instanceof Error) {
        return { name: val.name, message: val.message, stack: val.stack };
      }
      if (typeof val === "bigint") {
        return val.toString();
      }
      if (typeof val === "object" && val !== null) {
        if (seen.has(val)) return "[Circular]";
        seen.add(val);
      }
      return val;
    });
  } catch {
    return String(arg);
  }
}

function formatLogMessage(msg: string, args: unknown[]): string {
  if (args.length === 0) return msg;
  const serialized = args.map(serializeArg).join(" ");
  return `${msg} ${serialized}`;
}

/**
 * Real-time Logger Bridge forwarding frontend events directly to the Rust Backend terminal.
 */
export const feLog = {
  info: (msg: string, ...args: unknown[]) => {
    const formatted = formatLogMessage(msg, args);
    console.log(`[FE] ${formatted}`);
    invoke(IPC_COMMANDS.LOG_FRONTEND, { level: "info", message: formatted }).catch(() => {});
  },
  warn: (msg: string, ...args: unknown[]) => {
    const formatted = formatLogMessage(msg, args);
    console.warn(`[FE] ${formatted}`);
    invoke(IPC_COMMANDS.LOG_FRONTEND, { level: "warn", message: formatted }).catch(() => {});
  },
  error: (msg: string, ...args: unknown[]) => {
    const formatted = formatLogMessage(msg, args);
    console.error(`[FE] ${formatted}`);
    invoke(IPC_COMMANDS.LOG_FRONTEND, { level: "error", message: formatted }).catch(() => {});
  },
};
