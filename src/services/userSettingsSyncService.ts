import i18n from "../i18n";
import { useSettingsStore } from "../stores/settingsStore";
import { useMessengerStore } from "../platforms/messenger/stores/messengerStore";
import { createMessengerAccount } from "../platforms/messenger/types";
import { systemService } from "./systemService";
import { type UserSettingsSnapshot } from "../types/ipc";

interface SyncOptions {
  syncMissingUpdateMode?: boolean;
}

/**
 * Applies a backend SSOT settings snapshot (user_cache.json) onto frontend stores idempotently.
 */
export function applyBackendUserSettings(
  userSettings: UserSettingsSnapshot,
  options?: SyncOptions,
): void {
  const settingsStore = useSettingsStore.getState();
  const bubbleSize = userSettings.bubbleSize ?? userSettings.bubble_size;
  if ((bubbleSize === "sm" || bubbleSize === "md" || bubbleSize === "lg") && settingsStore.bubbleSize !== bubbleSize) {
    settingsStore.setBubbleSize(bubbleSize);
  }
  const positionMode = userSettings.bubblePositionMode ?? userSettings.bubble_position_mode;
  if ((positionMode === "auto_tuck" || positionMode === "free") && settingsStore.bubblePositionMode !== positionMode) {
    settingsStore.setBubblePositionMode(positionMode, true);
  }
  const alwaysOnTop = typeof userSettings.isAlwaysOnTop === "boolean" ? userSettings.isAlwaysOnTop : userSettings.is_always_on_top;
  if (typeof alwaysOnTop === "boolean" && settingsStore.isAlwaysOnTop !== alwaysOnTop) {
    settingsStore.setAlwaysOnTop(alwaysOnTop);
  }
  if ((userSettings.locale === "en" || userSettings.locale === "vi") && (settingsStore.language !== userSettings.locale || i18n.language !== userSettings.locale)) {
    settingsStore.setLanguage(userSettings.locale, true);
  }
  const autoTrim = typeof userSettings.isAutoTrimEnabled === "boolean" ? userSettings.isAutoTrimEnabled : userSettings.is_auto_trim_enabled;
  if (typeof autoTrim === "boolean" && settingsStore.isAutoTrimEnabled !== autoTrim) {
    settingsStore.setAutoTrimEnabled(autoTrim, true);
  }
  const themeMode = userSettings.themeMode ?? userSettings.theme_mode;
  if ((themeMode === "light" || themeMode === "dark" || themeMode === "system") && settingsStore.themeMode !== themeMode) {
    settingsStore.setThemeMode(themeMode, true);
  }
  const updateMode = userSettings.updateMode ?? userSettings.update_mode;
  if (updateMode === "auto" || updateMode === "manual") {
    if (settingsStore.updateMode !== updateMode) {
      settingsStore.setUpdateMode(updateMode, true);
    }
  } else if (options?.syncMissingUpdateMode) {
    systemService.setUpdateMode(settingsStore.updateMode);
  }
  const globalShortcutEnabled = typeof userSettings.isGlobalShortcutEnabled === "boolean" ? userSettings.isGlobalShortcutEnabled : userSettings.is_global_shortcut_enabled;
  if (typeof globalShortcutEnabled === "boolean" && settingsStore.isGlobalShortcutEnabled !== globalShortcutEnabled) {
    settingsStore.setGlobalShortcutEnabled(globalShortcutEnabled, true);
  }
  const authAccountsRaw = userSettings.authenticatedAccounts ?? userSettings.authenticated_accounts;
  const activeBubblesRaw = userSettings.activeBubbleAccounts ?? userSettings.active_bubble_accounts;
  const mutedAccountsRaw = userSettings.mutedAccounts ?? userSettings.muted_accounts;
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
  if (Array.isArray(activeBubblesRaw)) {
    const activeSet = new Set(activeBubblesRaw);
    const store = useMessengerStore.getState();
    for (const acc of store.accounts) {
      const shouldBeVisible = activeSet.has(acc.index);
      if (acc.isBubbleVisible !== shouldBeVisible) {
        store.setAccountBubbleVisibility(acc.index, shouldBeVisible);
      }
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
    const nextNotificationsEnabled = mutedAccountsRaw.length < store.accounts.length;
    if (settingsStore.isNotificationsEnabled !== nextNotificationsEnabled) {
      settingsStore.setNotificationsEnabled(nextNotificationsEnabled);
    }
  }
}
