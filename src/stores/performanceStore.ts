import { create } from "zustand";
import { SystemPerformanceSnapshot } from "../types/performance";
import { systemService } from "../services/systemService";

interface PerformanceState {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
  error: string | null;
  isPolling: boolean;
  pollingIntervalMs: number;
  fetchSnapshot: () => Promise<void>;
  startPolling: () => void;
  stopPolling: () => void;
  setPollingIntervalMs: (ms: number) => void;
}

let pollingTimer: ReturnType<typeof setTimeout> | null = null;
let isPollingActive = false;
let isFetchingSnapshot = false;
let pollingGeneration = 0;

export const usePerformanceStore = create<PerformanceState>((set, get) => ({
  snapshot: null,
  isLoading: false,
  error: null,
  isPolling: false,
  pollingIntervalMs: 1500,
  fetchSnapshot: async () => {
    if (isFetchingSnapshot) return;
    isFetchingSnapshot = true;
    try {
      if (!get().snapshot) {
        set({ isLoading: true });
      }
      const data = await systemService.getPerformanceSnapshot();
      if (data) {
        set({ snapshot: data, error: null, isLoading: false });
      } else {
        set({ error: "Failed to get system performance data", isLoading: false });
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      set({ error: msg, isLoading: false });
    } finally {
      isFetchingSnapshot = false;
    }
  },
  startPolling: () => {
    if (isPollingActive) return;
    isPollingActive = true;
    const gen = ++pollingGeneration;
    set({ isPolling: true });
    const runLoop = async () => {
      await get().fetchSnapshot();
      if (!isPollingActive || pollingGeneration !== gen) return;
      pollingTimer = setTimeout(runLoop, get().pollingIntervalMs);
    };
    runLoop();
  },
  stopPolling: () => {
    isPollingActive = false;
    pollingGeneration++;
    if (pollingTimer !== null) {
      clearTimeout(pollingTimer);
      pollingTimer = null;
    }
    set({ isPolling: false });
  },
  setPollingIntervalMs: (ms: number) => {
    set({ pollingIntervalMs: ms });
    if (isPollingActive) {
      get().stopPolling();
      get().startPolling();
    }
  },
}));
