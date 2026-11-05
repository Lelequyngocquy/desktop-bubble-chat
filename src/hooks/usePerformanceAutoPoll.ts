import { useEffect } from "react";
import { usePerformanceStore } from "../stores/performanceStore";

/**
 * Hook to automatically enable adaptive performance polling when mounted
 * and clean up when unmounted.
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
