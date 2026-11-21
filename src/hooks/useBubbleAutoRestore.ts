import { useCallback } from "react";
import { useMessengerStore, messengerService } from "../platforms/messenger";
import { feLog } from "../services/loggerService";

/**
 * Exposes manual helper to re-sync bubble visibility state from Backend SSOT.
 */
export function useBubbleAutoRestore() {
  const syncActiveBubbles = useCallback(async () => {
    try {
      const activeIndices = await messengerService.getActiveBubbleAccounts();
      const activeSet = new Set(activeIndices);
      const store = useMessengerStore.getState();
      for (const acc of store.accounts) {
        const isLive = activeSet.has(acc.index);
        if (acc.isBubbleVisible !== isLive) {
          feLog.info(
            `[useBubbleAutoRestore] Syncing Bubble ${acc.index} state from Backend SSOT: ${isLive ? "ON" : "OFF"}`,
          );
          store.setAccountBubbleVisibility(acc.index, isLive);
        }
      }
    } catch (err) {
      feLog.warn("[useBubbleAutoRestore] Error querying active bubble snapshot:", err);
    }
  }, []);
  return { syncActiveBubbles };
}

export default useBubbleAutoRestore;
