import { create } from "zustand";
import { useEffect } from "react";
import { SystemPerformanceSnapshot } from "../types/performance";
import { systemService } from "../services/systemService";

interface PerformanceState {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
  isTrimming: boolean;
  lastTrimmedMb: number | null;
  error: string | null;
  isPolling: boolean;
  pollingIntervalMs: number;

  // Actions
  fetchSnapshot: () => Promise<void>;
  startPolling: () => void;
  stopPolling: () => void;
  trimMemory: () => Promise<number>;
  setPollingIntervalMs: (ms: number) => void;
}

let pollingTimer: ReturnType<typeof setInterval> | null = null;

export const usePerformanceStore = create<PerformanceState>((set, get) => ({
  snapshot: null,
  isLoading: false,
  isTrimming: false,
  lastTrimmedMb: null,
  error: null,
  isPolling: false,
  pollingIntervalMs: 1500,

  fetchSnapshot: async () => {
    try {
      if (!get().snapshot) {
        set({ isLoading: true });
      }

      const data = await systemService.getPerformanceSnapshot();
      if (data) {
        set({ snapshot: data, error: null, isLoading: false });
      } else {
        set({ error: "Không thể lấy dữ liệu hiệu năng từ hệ thống", isLoading: false });
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      set({ error: msg, isLoading: false });
    }
  },

  startPolling: () => {
    if (pollingTimer !== null) {
      return;
    }

    set({ isPolling: true });

    // Quét ngay lập tức lần đầu tiên
    get().fetchSnapshot();

    // Thiết lập timer định kỳ
    const interval = get().pollingIntervalMs;
    pollingTimer = setInterval(() => {
      get().fetchSnapshot();
    }, interval);
  },

  stopPolling: () => {
    if (pollingTimer !== null) {
      clearInterval(pollingTimer);
      pollingTimer = null;
    }
    set({ isPolling: false });
  },

  trimMemory: async () => {
    try {
      set({ isTrimming: true });
      const freedMb = await systemService.trimSystemMemory();
      set({ lastTrimmedMb: freedMb, isTrimming: false });

      // Cập nhật lại snapshot sau khi trim RAM
      await get().fetchSnapshot();
      return freedMb;
    } catch (err) {
      set({ isTrimming: false });
      console.error("[PerformanceStore] Lỗi dọn dẹp RAM:", err);
      return 0;
    }
  },

  setPollingIntervalMs: (ms: number) => {
    set({ pollingIntervalMs: ms });
    if (get().isPolling) {
      get().stopPolling();
      get().startPolling();
    }
  },
}));

/**
 * Hook tự động bật Adaptive Polling khi component (Tab Hiệu năng) mount
 * và tự động tắt hoàn toàn timer khi unmount để đạt 0% chi phí nền.
 */
export function usePerformanceAutoPoll() {
  const startPolling = usePerformanceStore((s) => s.startPolling);
  const stopPolling = usePerformanceStore((s) => s.stopPolling);

  useEffect(() => {
    startPolling();
    return () => {
      stopPolling();
    };
  }, [startPolling, stopPolling]);
}
