import { create } from "zustand";

/**
 * Dashboard lifecycle states:
 * - initializing: App launched or Dashboard restored from hibernation
 * - rehydrating: Hydrating configuration, sessions, and data
 * - ready: Ready for user interaction
 */
export type DashboardLifecycleStatus = "initializing" | "rehydrating" | "ready";

interface AppLifecycleState {
  status: DashboardLifecycleStatus;
  isDashboardReady: boolean;
  statusMessage: string;
  setDashboardStatus: (status: DashboardLifecycleStatus, message?: string) => void;
  markDashboardReady: () => void;
  resetDashboardState: (status?: DashboardLifecycleStatus, message?: string) => void;
}

export const useAppLifecycleStore = create<AppLifecycleState>((set) => ({
  status: "initializing",
  isDashboardReady: false,
  statusMessage: "Initializing workspace...",

  setDashboardStatus: (status, message) =>
    set((prev) => ({
      status,
      isDashboardReady: status === "ready",
      statusMessage: message ?? (status === "ready" ? "Ready" : prev.statusMessage),
    })),

  markDashboardReady: () =>
    set({
      status: "ready",
      isDashboardReady: true,
      statusMessage: "Ready",
    }),

  resetDashboardState: (status = "initializing", message = "Synchronizing workspace...") =>
    set({
      status,
      isDashboardReady: false,
      statusMessage: message,
    }),
}));