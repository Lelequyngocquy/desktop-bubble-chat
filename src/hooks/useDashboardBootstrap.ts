import { useEffect, useRef } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppLifecycleStore } from "../stores/appLifecycleStore";
import { useSystemUserStore } from "../stores/systemUserStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useUpdateStore } from "../stores/updateStore";
import { useMessengerStore } from "../platforms/messenger/stores/messengerStore";
import { createMessengerAccount } from "../platforms/messenger/types";
import { systemService } from "../services/systemService";
import { messengerService } from "../platforms/messenger/services/messengerService";
import { feLog } from "../services/loggerService";
import { safeListen } from "../utils/tauriEvents";
import { IPC_EVENTS } from "../types/ipc";

/**
 * Single startup bootstrap coordinator hydrating user settings, version, accounts, and bubble states.
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
        const [usernameRes, autostartRes, settingsRes, versionRes] = await Promise.allSettled([
          systemService.getSystemUsername(),
          systemService.isAutostartEnabled(),
          systemService.getUserSettings(),
          getVersion(),
        ]);
        if (usernameRes.status === "fulfilled") {
          useSystemUserStore.getState().setSystemUser(usernameRes.value);
        } else {
          feLog.warn("[useDashboardBootstrap] Unable to get system username:", usernameRes.reason);
        }
        if (autostartRes.status === "fulfilled") {
          useSettingsStore.getState().setAutoStart(autostartRes.value);
        } else {
          feLog.warn("[useDashboardBootstrap] Unable to check autostart:", autostartRes.reason);
        }
        if (versionRes.status === "fulfilled" && versionRes.value) {
          useUpdateStore.getState().setCurrentVersion(versionRes.value);
        }
        if (settingsRes.status === "fulfilled" && settingsRes.value) {
          const userSettings = settingsRes.value;
          const settingsStore = useSettingsStore.getState();
          const bubbleSize = userSettings.bubbleSize || userSettings.bubble_size;
          if (bubbleSize && (bubbleSize === "sm" || bubbleSize === "md" || bubbleSize === "lg")) {
            settingsStore.setBubbleSize(bubbleSize);
          }
          const positionMode = userSettings.bubblePositionMode || userSettings.bubble_position_mode;
          if (positionMode && (positionMode === "auto_tuck" || positionMode === "free")) {
            settingsStore.setBubblePositionMode(positionMode, true);
          }
          const alwaysOnTop = typeof userSettings.isAlwaysOnTop === "boolean" ? userSettings.isAlwaysOnTop : userSettings.is_always_on_top;
          if (typeof alwaysOnTop === "boolean") {
            settingsStore.setAlwaysOnTop(alwaysOnTop);
          }
          if (userSettings.locale && (userSettings.locale === "en" || userSettings.locale === "vi")) {
            settingsStore.setLanguage(userSettings.locale, true);
          }
          const autoTrim = typeof userSettings.isAutoTrimEnabled === "boolean" ? userSettings.isAutoTrimEnabled : userSettings.is_auto_trim_enabled;
          if (typeof autoTrim === "boolean") {
            settingsStore.setAutoTrimEnabled(autoTrim, true);
          }
          const themeMode = userSettings.themeMode || userSettings.theme_mode;
          if (themeMode && (themeMode === "light" || themeMode === "dark" || themeMode === "system")) {
            settingsStore.setThemeMode(themeMode, true);
          }
          const updateMode = userSettings.updateMode || userSettings.update_mode;
          if (updateMode && (updateMode === "auto" || updateMode === "manual")) {
            settingsStore.setUpdateMode(updateMode, true);
          } else {
            systemService.setUpdateMode(settingsStore.updateMode);
          }
          const globalShortcutEnabled = typeof userSettings.isGlobalShortcutEnabled === "boolean" ? userSettings.isGlobalShortcutEnabled : userSettings.is_global_shortcut_enabled;
          if (typeof globalShortcutEnabled === "boolean") {
            settingsStore.setGlobalShortcutEnabled(globalShortcutEnabled, true);
          }
          const authAccountsRaw = userSettings.authenticatedAccounts || userSettings.authenticated_accounts;
          const mutedAccountsRaw = userSettings.mutedAccounts || userSettings.muted_accounts;
          if (Array.isArray(authAccountsRaw)) {
            const store = useMessengerStore.getState();
            const existingIndices = new Set(store.accounts.map((a) => a.index));
            const extraAccounts = authAccountsRaw
              .filter((idx): idx is number => typeof idx === "number" && Number.isInteger(idx) && idx >= 1 && idx <= 5 && !existingIndices.has(idx))
              .map((idx) => createMessengerAccount(idx));
            if (extraAccounts.length > 0) {
              store.setAccounts([...store.accounts, ...extraAccounts].sort((a, b) => a.index - b.index));
            }
          }
          if (Array.isArray(mutedAccountsRaw)) {
            const mutedSet = new Set(mutedAccountsRaw);
            const store = useMessengerStore.getState();
            for (const acc of store.accounts) {
              const shouldEnableSound = !mutedSet.has(acc.index);
              if ((acc.soundEnabled ?? true) !== shouldEnableSound) {
                store.updateAccount(acc.id, { soundEnabled: shouldEnableSound });
              }
            }
            settingsStore.setNotificationsEnabled(mutedAccountsRaw.length < store.accounts.length);
          }
        } else if (settingsRes.status === "rejected") {
          feLog.warn("[useDashboardBootstrap] Unable to sync user settings from cache:", settingsRes.reason);
        }
        try {
          const accounts = useMessengerStore.getState().accounts;
          await Promise.all(
            accounts.map(async (acc) => {
              try {
                const res = await messengerService.checkAuth(acc.index);
                if (res && typeof res.connected === "boolean") {
                  useMessengerStore.getState().updateAccount(acc.id, {
                    isConnected: res.connected,
                    isAuthenticating: false,
                    isBubbleVisible: res.connected ? acc.isBubbleVisible : false,
                  });
                }
              } catch (err) {
                feLog.warn(`[useDashboardBootstrap] Auth check skipped for account ${acc.index}:`, err);
              }
            }),
          );
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
        try {
          const chatSettings = useSettingsStore.getState().chatWindowSettings;
          for (const s of chatSettings) {
            if (s.mode === "remember_custom" && s.customWidth && s.customHeight) {
              await messengerService.setChatWindowSizeConfig(s.accountIndex, true, s.customWidth, s.customHeight);
            }
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Unable to sync chat window size configs:", err);
        }
        feLog.info("[useDashboardBootstrap] Bootstrap completed! Dashboard is ready.");
        markDashboardReady();
      } catch (err) {
        feLog.error("[useDashboardBootstrap] Unhandled error during bootstrap:", err);
        markDashboardReady();
      } finally {
        clearTimeout(safetyTimer);
      }
    };
    let disposed = false;
    let unlistenLang: (() => void) | undefined;
    safeListen<string>(
      IPC_EVENTS.LANGUAGE_CHANGED,
      (event) => {
        const newLang = event.payload;
        if (newLang === "en" || newLang === "vi") {
          useSettingsStore.getState().setLanguage(newLang, true);
        }
      },
      () => disposed,
      (unsub) => {
        unlistenLang = unsub;
      },
    ).catch(() => {});
    runBootstrap();
    return () => {
      disposed = true;
      clearTimeout(safetyTimer);
      unlistenLang?.();
    };
  }, [setDashboardStatus, markDashboardReady]);
  return { isDashboardReady };
}

export default useDashboardBootstrap;
