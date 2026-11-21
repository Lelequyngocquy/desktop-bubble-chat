import { create } from "zustand";
import { persist } from "zustand/middleware";
import i18n from "../i18n";
import { systemService } from "../services/systemService";
import { messengerService } from "../platforms/messenger/services/messengerService";

export type BubbleSize = "sm" | "md" | "lg";
export type BubblePositionMode = "auto_tuck" | "free";
export type AppLanguage = "vi" | "en";
export type ThemeMode = "light" | "dark" | "system";
export type CloseWindowAction = "ask" | "hide_to_tray" | "exit_app";
export type UpdateMode = "auto" | "manual";
export type ChatWindowSizeMode = "default" | "remember_custom";

export interface ChatWindowSizeSetting {
  accountId: string;
  accountIndex: number;
  mode: ChatWindowSizeMode;
  customWidth?: number;
  customHeight?: number;
}

export const DEFAULT_CHAT_WIDTH = 404;
export const DEFAULT_CHAT_HEIGHT = 550;
export const MIN_CHAT_WIDTH = 320;
export const MIN_CHAT_HEIGHT = 450;

export const BUBBLE_SIZE_PIXELS: Record<BubbleSize, number> = {
  sm: 48,
  md: 56,
  lg: 64,
};

export function applyThemeMode(mode: ThemeMode) {
  if (typeof document === "undefined") return;
  const isDark =
    mode === "dark" ||
    (mode === "system" &&
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);
  if (isDark) {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}

interface SettingsState {
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  isAutoStart: boolean;
  isGlobalShortcutEnabled: boolean;
  bubbleSize: BubbleSize;
  bubblePositionMode: BubblePositionMode;
  language: AppLanguage;
  isAutoTrimEnabled: boolean;
  themeMode: ThemeMode;
  closeWindowAction: CloseWindowAction;
  updateMode: UpdateMode;
  chatWindowSettings: ChatWindowSizeSetting[];
  setAlwaysOnTop: (enabled: boolean) => void;
  toggleAlwaysOnTop: () => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  toggleNotifications: () => void;
  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;
  setGlobalShortcutEnabled: (enabled: boolean, skipIpc?: boolean) => void;
  toggleGlobalShortcut: () => void;
  setBubbleSize: (size: BubbleSize) => void;
  setBubblePositionMode: (mode: BubblePositionMode, skipIpc?: boolean) => void;
  setLanguage: (lang: AppLanguage, skipIpc?: boolean) => void;
  setAutoTrimEnabled: (enabled: boolean, skipIpc?: boolean) => void;
  toggleAutoTrim: () => void;
  setThemeMode: (mode: ThemeMode, skipIpc?: boolean) => void;
  setCloseWindowAction: (action: CloseWindowAction) => void;
  setUpdateMode: (mode: UpdateMode, skipIpc?: boolean) => void;
  setChatWindowMode: (accountIndex: number, accountId: string, mode: ChatWindowSizeMode) => void | Promise<void>;
  updateChatWindowSize: (accountIndex: number, width: number, height: number) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      isAlwaysOnTop: true,
      isNotificationsEnabled: true,
      isAutoStart: false,
      isGlobalShortcutEnabled: true,
      bubbleSize: "md",
      bubblePositionMode: "auto_tuck",
      language: "en",
      isAutoTrimEnabled: true,
      themeMode: "system",
      chatWindowSettings: [],
      updateMode: "auto",
      closeWindowAction: "ask",
      setAlwaysOnTop: (enabled) => set({ isAlwaysOnTop: enabled }),
      toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),
      setNotificationsEnabled: (enabled) => set({ isNotificationsEnabled: enabled }),
      toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),
      setAutoStart: (enabled) => set({ isAutoStart: enabled }),
      toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),
      setGlobalShortcutEnabled: (enabled, skipIpc = false) => {
        set({ isGlobalShortcutEnabled: enabled });
        if (!skipIpc) {
          systemService.setGlobalShortcutEnabled(enabled);
        }
      },
      toggleGlobalShortcut: () => {
        const next = !get().isGlobalShortcutEnabled;
        set({ isGlobalShortcutEnabled: next });
        systemService.setGlobalShortcutEnabled(next);
      },
      setBubbleSize: (size) => set({ bubbleSize: size }),
      setBubblePositionMode: (mode, skipIpc = false) => {
        set({ bubblePositionMode: mode });
        if (!skipIpc) {
          systemService.setBubblePositionMode(mode);
        }
      },
      setLanguage: (lang, skipIpc = false) => {
        set({ language: lang });
        i18n.changeLanguage(lang);
        if (!skipIpc) {
          systemService.setAppLanguage(lang);
        }
      },
      setAutoTrimEnabled: (enabled, skipIpc = false) => {
        set({ isAutoTrimEnabled: enabled });
        if (!skipIpc) {
          systemService.setAutoTrimEnabled(enabled);
        }
      },
      toggleAutoTrim: () => {
        const next = !get().isAutoTrimEnabled;
        set({ isAutoTrimEnabled: next });
        systemService.setAutoTrimEnabled(next);
      },
      setThemeMode: (mode, skipIpc = false) => {
        set({ themeMode: mode });
        applyThemeMode(mode);
        if (!skipIpc) {
          systemService.setThemeMode(mode);
        }
      },
      setCloseWindowAction: (action) => set({ closeWindowAction: action }),
      setUpdateMode: (mode, skipIpc = false) => {
        set({ updateMode: mode });
        if (!skipIpc) {
          systemService.setUpdateMode(mode);
        }
      },
      setChatWindowMode: async (accountIndex, accountId, mode) => {
        set((state) => {
          const existing = state.chatWindowSettings.find((s) => s.accountIndex === accountIndex);
          if (existing) {
            return {
              chatWindowSettings: state.chatWindowSettings.map((s) =>
                s.accountIndex === accountIndex ? { ...s, mode } : s,
              ),
            };
          }
          return {
            chatWindowSettings: [
              ...state.chatWindowSettings,
              { accountId, accountIndex, mode, customWidth: DEFAULT_CHAT_WIDTH, customHeight: DEFAULT_CHAT_HEIGHT },
            ],
          };
        });
        const isCustom = mode === "remember_custom";
        const current = get().chatWindowSettings.find((s) => s.accountIndex === accountIndex);
        const res = await messengerService.setChatWindowSizeConfig(
          accountIndex,
          isCustom,
          isCustom && current?.customWidth ? current.customWidth : undefined,
          isCustom && current?.customHeight ? current.customHeight : undefined,
        );
        if (isCustom && res && Array.isArray(res) && res.length === 2) {
          const [w, h] = res;
          if (w >= MIN_CHAT_WIDTH && h >= MIN_CHAT_HEIGHT) {
            set((state) => ({
              chatWindowSettings: state.chatWindowSettings.map((s) =>
                s.accountIndex === accountIndex ? { ...s, customWidth: w, customHeight: h } : s,
              ),
            }));
          }
        }
      },
      updateChatWindowSize: (accountIndex, width, height) => {
        const clampedWidth = Math.max(MIN_CHAT_WIDTH, Math.round(width));
        const clampedHeight = Math.max(MIN_CHAT_HEIGHT, Math.round(height));
        set((state) => {
          const existing = state.chatWindowSettings.find((s) => s.accountIndex === accountIndex);
          if (existing) {
            if (
              existing.customWidth !== undefined &&
              existing.customHeight !== undefined &&
              Math.abs(existing.customWidth - clampedWidth) < 2 &&
              Math.abs(existing.customHeight - clampedHeight) < 2
            ) {
              return state;
            }
            return {
              chatWindowSettings: state.chatWindowSettings.map((s) =>
                s.accountIndex === accountIndex ? { ...s, customWidth: clampedWidth, customHeight: clampedHeight } : s,
              ),
            };
          }
          return {
            chatWindowSettings: [
              ...state.chatWindowSettings,
              {
                accountId: `messenger-${accountIndex.toString().padStart(2, "0")}`,
                accountIndex,
                mode: "remember_custom",
                customWidth: clampedWidth,
                customHeight: clampedHeight,
              },
            ],
          };
        });
        messengerService.setChatWindowSizeConfig(accountIndex, true, clampedWidth, clampedHeight);
      },
    }),
    {
      name: "desktop_bubble_chat_settings",
      version: 1,
      migrate: (persistedState: unknown, version: number) => {
        const state = (persistedState && typeof persistedState === "object" ? { ...(persistedState as Record<string, unknown>) } : {}) as Record<string, unknown>;
        if (version < 1 && typeof localStorage !== "undefined") {
          try {
            const legacyClose = localStorage.getItem("dbc_close_window_action");
            if ((legacyClose === "hide_to_tray" || legacyClose === "exit_app") && !state.closeWindowAction) {
              state.closeWindowAction = legacyClose;
            }
            localStorage.removeItem("dbc_close_window_action");
            const oldUpdate = localStorage.getItem("desktop_bubble_chat_update_settings");
            if (oldUpdate && !state.updateMode) {
              const parsed = JSON.parse(oldUpdate);
              if (parsed?.state?.updateMode === "auto" || parsed?.state?.updateMode === "manual") {
                state.updateMode = parsed.state.updateMode;
              }
            }
          } catch {}
        }
        return state as unknown as SettingsState;
      },
      onRehydrateStorage: () => (state) => {
        if (state?.language) {
          i18n.changeLanguage(state.language);
        }
        if (state?.themeMode) {
          applyThemeMode(state.themeMode);
        }
      },
    },
  ),
);

type PersistedSettingsPatch = Partial<
  Pick<
    SettingsState,
    | "isAlwaysOnTop"
    | "isNotificationsEnabled"
    | "isAutoStart"
    | "isGlobalShortcutEnabled"
    | "bubbleSize"
    | "bubblePositionMode"
    | "language"
    | "isAutoTrimEnabled"
    | "themeMode"
    | "closeWindowAction"
    | "updateMode"
    | "chatWindowSettings"
  >
>;

function extractValidPersistedSettings(raw: unknown): PersistedSettingsPatch {
  if (!raw || typeof raw !== "object") return {};
  const src = raw as Record<string, unknown>;
  const patch: PersistedSettingsPatch = {};
  if (typeof src.isAlwaysOnTop === "boolean") patch.isAlwaysOnTop = src.isAlwaysOnTop;
  if (typeof src.isNotificationsEnabled === "boolean") patch.isNotificationsEnabled = src.isNotificationsEnabled;
  if (typeof src.isAutoStart === "boolean") patch.isAutoStart = src.isAutoStart;
  if (typeof src.isGlobalShortcutEnabled === "boolean") patch.isGlobalShortcutEnabled = src.isGlobalShortcutEnabled;
  if (typeof src.isAutoTrimEnabled === "boolean") patch.isAutoTrimEnabled = src.isAutoTrimEnabled;
  if (src.bubbleSize === "sm" || src.bubbleSize === "md" || src.bubbleSize === "lg") patch.bubbleSize = src.bubbleSize;
  if (src.bubblePositionMode === "auto_tuck" || src.bubblePositionMode === "free") patch.bubblePositionMode = src.bubblePositionMode;
  if (src.language === "en" || src.language === "vi") patch.language = src.language;
  if (src.themeMode === "light" || src.themeMode === "dark" || src.themeMode === "system") patch.themeMode = src.themeMode;
  if (src.closeWindowAction === "ask" || src.closeWindowAction === "hide_to_tray" || src.closeWindowAction === "exit_app") patch.closeWindowAction = src.closeWindowAction;
  if (src.updateMode === "auto" || src.updateMode === "manual") patch.updateMode = src.updateMode;
  if (Array.isArray(src.chatWindowSettings)) {
    patch.chatWindowSettings = src.chatWindowSettings.filter((item): item is ChatWindowSizeSetting => {
      if (!item || typeof item !== "object") return false;
      const entry = item as Partial<ChatWindowSizeSetting>;
      return (
        typeof entry.accountId === "string" &&
        typeof entry.accountIndex === "number" &&
        Number.isInteger(entry.accountIndex) &&
        entry.accountIndex >= 1 &&
        entry.accountIndex <= 5 &&
        (entry.mode === "default" || entry.mode === "remember_custom")
      );
    });
  }
  return patch;
}

if (typeof window !== "undefined") {
  window.addEventListener("storage", (event) => {
    if (event.key === "desktop_bubble_chat_settings" && event.newValue) {
      try {
        const parsed = JSON.parse(event.newValue);
        const validated = extractValidPersistedSettings(parsed?.state);
        if (Object.keys(validated).length > 0) {
          useSettingsStore.setState(validated);
          if (validated.language) {
            i18n.changeLanguage(validated.language);
          }
          if (validated.themeMode) {
            applyThemeMode(validated.themeMode);
          }
        }
      } catch (err) {
        console.warn("[SettingsStore] Failed to sync storage event:", err);
      }
    }
  });
}
