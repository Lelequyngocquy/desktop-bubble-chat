import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMessengerStore, messengerService } from "../platforms/messenger";
import { useAppLifecycleStore } from "../stores/appLifecycleStore";
import { feLog } from "../services/loggerService";

/**
 * Syncs bubble visibility state with Backend SSOT snapshot
 */
export function useBubbleAutoRestore() {
  const isDashboardReady = useAppLifecycleStore((s) => s.isDashboardReady);

  useEffect(() => {
    let isMainWindow = true;
    try {
      const label = getCurrentWindow()?.label;
      if (label && label !== "main") {
        isMainWindow = false;
      }
    } catch {
      isMainWindow = true;
    }
    if (!isMainWindow) return;
    if (!isDashboardReady) return;
    messengerService
      .getActiveBubbleAccounts()
      .then((activeIndices) => {
        const activeSet = new Set(activeIndices);
        const store = useMessengerStore.getState();
        for (const acc of store.accounts) {
          const isLive = activeSet.has(acc.index);
          if (acc.isBubbleVisible !== isLive) {
            feLog.info(
              `[useBubbleAutoRestore] Syncing Bubble ${acc.index} state from Backend SSOT: ${isLive ? "ON" : "OFF"}`
            );
            store.setAccountBubbleVisibility(acc.index, isLive);
          }
        }
      })
      .catch((err) => {
        feLog.warn("[useBubbleAutoRestore] Error querying active bubble snapshot:", err);
      });
  }, [isDashboardReady]);
}

export default useBubbleAutoRestore;