import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMessengerStore } from "../platforms/messenger";
import { systemService } from "../services/systemService";
import { safeListen } from "../utils/tauriEvents";
import { registerMessengerEvents } from "./unread/registerMessengerEvents";
import { registerWindowAndLayoutEvents } from "./unread/registerWindowAndLayoutEvents";

/**
 * Global background listener synchronizing Tauri desktop IPC events with Domain Stores.
 */
export const useUnreadListener = () => {
  const prevUnreadMap = useRef<Map<number, number>>(new Map());
  const isInitialGracePeriod = useRef<boolean>(true);
  useEffect(() => {
    try {
      const currentWin = getCurrentWindow();
      if (currentWin && currentWin.label && currentWin.label !== "main") {
        return;
      }
    } catch (err) {
      console.warn("[useUnreadListener] Could not resolve window label:", err);
    }
    if (
      typeof window !== "undefined" &&
      (window.location.search.includes("window=bubble") ||
        window.location.href.includes("bubble"))
    ) {
      return;
    }
    let disposed = false;
    const unlisteners: Array<() => void> = [];
    const timerCleanups: Array<() => void> = [];
    const graceTimer = setTimeout(() => {
      isInitialGracePeriod.current = false;
    }, 3500);
    const register = async <T>(eventName: string, onPayload: (payload: T) => void): Promise<boolean> => {
      if (disposed) return false;
      await safeListen<T>(
        eventName,
        (event) => {
          if (!disposed) onPayload(event.payload);
        },
        () => disposed,
        (fn) => unlisteners.push(fn),
      );
      return !disposed;
    };
    const setupListeners = async () => {
      try {
        const currentAccounts = useMessengerStore.getState().accounts;
        systemService.updateTrayMenu(currentAccounts.some((a) => a.isConnected));
        const ctx = {
          register,
          isDisposed: () => disposed,
          prevUnreadMap: prevUnreadMap.current,
          isInitialGracePeriod,
        };
        if (!(await registerMessengerEvents(ctx))) return;
        await registerWindowAndLayoutEvents(ctx, (cleanup) => timerCleanups.push(cleanup));
      } catch (err) {
        console.warn("[useUnreadListener] Desktop IPC event listeners not available in preview:", err);
      }
    };
    const unsubscribeStore = useMessengerStore.subscribe((state, prevState) => {
      const wasConnected = prevState.accounts.some((a) => a.isConnected);
      const isConnected = state.accounts.some((a) => a.isConnected);
      if (wasConnected !== isConnected) {
        systemService.updateTrayMenu(isConnected);
      }
    });
    setupListeners();
    return () => {
      disposed = true;
      clearTimeout(graceTimer);
      timerCleanups.forEach((cleanup) => cleanup());
      unsubscribeStore();
      unlisteners.forEach((unsub) => unsub());
    };
  }, []);
  const simulateUnread = async (count: number) => {
    const activeIndex = useMessengerStore.getState().activeAccountIndex;
    useMessengerStore.getState().setAccountUnreadCount(activeIndex, count);
    await systemService.setBadgeCount(count);
  };
  return { simulateUnread };
};
