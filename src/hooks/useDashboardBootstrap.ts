import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useAppLifecycleStore } from "../stores/appLifecycleStore";
import { useSystemUserStore } from "../stores/systemUserStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useMessengerStore } from "../platforms/messenger/stores/messengerStore";
import { systemService } from "../services/systemService";
import { messengerService } from "../platforms/messenger/services/messengerService";
import { feLog } from "../services/loggerService";
import { IPC_EVENTS } from "../types/ipc";

/**
 * Coordinates bootstrap lifecycle and initial state hydration for main Dashboard
 */
export function useDashboardBootstrap() {
  const isBootstrappingRef = useRef(false);
  const setDashboardStatus = useAppLifecycleStore((s) => s.setDashboardStatus);
  const markDashboardReady = useAppLifecycleStore((s) => s.markDashboardReady);
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
    if (isBootstrappingRef.current) return;
    isBootstrappingRef.current = true;
    feLog.info("[useDashboardBootstrap] Starting Dashboard bootstrap...");
    setDashboardStatus("initializing");
    const safetyTimer = setTimeout(() => {
      if (!useAppLifecycleStore.getState().isDashboardReady) {
        feLog.warn("[useDashboardBootstrap] [Safety 3.5s] Unlocking Dashboard due to bootstrap timeout.");
        markDashboardReady();
      }
    }, 3500);
    const runBootstrap = async () => {
      try {
        setDashboardStatus("rehydrating");
        try {
          const username = await systemService.getSystemUsername();
          useSystemUserStore.getState().setSystemUser(username);
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Unable to get system username:", err);
        }
        try {
          const autostart = await systemService.isAutostartEnabled();
          useSettingsStore.getState().setAutoStart(autostart);
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Unable to check autostart:", err);
        }
        try {
          const userSettings = await systemService.getUserSettings();
          if (userSettings) {
            const bubbleSize = userSettings.bubbleSize || userSettings.bubble_size;
            if (bubbleSize && (bubbleSize === "sm" || bubbleSize === "md" || bubbleSize === "lg")) {
              useSettingsStore.getState().setBubbleSize(bubbleSize);
            }
            const alwaysOnTop = typeof userSettings.isAlwaysOnTop === "boolean" ? userSettings.isAlwaysOnTop : userSettings.is_always_on_top;
            if (typeof alwaysOnTop === "boolean") {
              useSettingsStore.getState().setAlwaysOnTop(alwaysOnTop);
            }
            if (userSettings.locale && (userSettings.locale === "en" || userSettings.locale === "vi")) {
              useSettingsStore.getState().setLanguage(userSettings.locale, true);
            }
            const autoTrim = typeof userSettings.isAutoTrimEnabled === "boolean" ? userSettings.isAutoTrimEnabled : userSettings.is_auto_trim_enabled;
            if (typeof autoTrim === "boolean") {
              useSettingsStore.getState().setAutoTrimEnabled(autoTrim, true);
            }
            const themeMode = userSettings.themeMode || userSettings.theme_mode;
            if (themeMode && (themeMode === "light" || themeMode === "dark" || themeMode === "system")) {
              useSettingsStore.getState().setThemeMode(themeMode as any, true);
            }
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Unable to sync user settings from cache:", err);
        }
        setDashboardStatus("rehydrating");
        try {
          const accounts = useMessengerStore.getState().accounts;
          for (const acc of accounts) {
            try {
              const res = await messengerService.checkAuth(acc.index);
              if (res && res.connected) {
                useMessengerStore.getState().updateAccount(acc.id, {
                  isConnected: true,
                });
              }
            } catch {}
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Unable to sync Messenger accounts:", err);
        }
        try {
          const activeIndices = await messengerService.getActiveBubbleAccounts();
          const activeSet = new Set(activeIndices);
          const store = useMessengerStore.getState();
          for (const acc of store.accounts) {
            const isLive = activeSet.has(acc.index);
            if (acc.isBubbleVisible !== isLive) {
              store.setAccountBubbleVisibility(acc.index, isLive);
            }
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Unable to get active bubbles snapshot:", err);
        }
        await new Promise((resolve) => setTimeout(resolve, 200));
        feLog.info("[useDashboardBootstrap] Bootstrap completed! Dashboard is ready.");
        markDashboardReady();
      } catch (err) {
        feLog.error("[useDashboardBootstrap] Unhandled error during bootstrap:", err);
        markDashboardReady();
      } finally {
        clearTimeout(safetyTimer);
      }
    };
    let unlistenLang: (() => void) | undefined;
    listen<string>(IPC_EVENTS.LANGUAGE_CHANGED, (event) => {
      const newLang = event.payload;
      if (newLang === "en" || newLang === "vi") {
        useSettingsStore.getState().setLanguage(newLang, true);
      }
    }).then((unsub) => {
      unlistenLang = unsub;
    }).catch(() => {});
    runBootstrap();
    return () => {
      clearTimeout(safetyTimer);
      unlistenLang?.();
    };
  }, [setDashboardStatus, markDashboardReady]);

  return { isDashboardReady };
}

export default useDashboardBootstrap;