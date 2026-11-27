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
  setChatWindowSettingsFromBackend: (settings: ChatWindowSizeSetting[]) => void;
  updateChatWindowSize: (accountIndex: number, width: number, height: number, skipIpc?: boolean) => void;
}

let legacyChatWindowSettingsForMigration: ChatWindowSizeSetting[] = [];

export function consumeLegacyChatWindowSettings(): ChatWindowSizeSetting[] {
  const pending = legacyChatWindowSettingsForMigration;
  legacyChatWindowSettingsForMigration = [];
  return pending;
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
      setChatWindowSettingsFromBackend: (settings) => set({ chatWindowSettings: settings }),
      updateChatWindowSize: (accountIndex, width, height, skipIpc = false) => {
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
        if (!skipIpc) {
          messengerService.setChatWindowSizeConfig(accountIndex, true, clampedWidth, clampedHeight);
        }
      },
    }),
    {
      name: "desktop_bubble_chat_settings",
      version: 3,
      partialize: (state) => ({
        closeWindowAction: state.closeWindowAction,
      }),
      migrate: (persistedState: unknown, version: number) => {
        const state = (persistedState && typeof persistedState === "object" ? { ...(persistedState as Record<string, unknown>) } : {}) as Record<string, unknown>;
        if (version < 1 && typeof localStorage !== "undefined") {
          try {
            const legacyClose = localStorage.getItem("dbc_close_window_action");
            if ((legacyClose === "hide_to_tray" || legacyClose === "exit_app") && !state.closeWindowAction) {
              state.closeWindowAction = legacyClose;
            }
            localStorage.removeItem("dbc_close_window_action");
          } catch (err) {
            console.warn("[settingsStore] Legacy settings migration skipped:", err);
          }
        }
        if (version < 3) {
          const legacyChatSettings = extractValidLegacyChatWindowSettings(state.chatWindowSettings);
          if (legacyChatSettings.length > 0) {
            legacyChatWindowSettingsForMigration = legacyChatSettings;
          }
        }
        return extractValidPersistedSettings(state) as unknown as SettingsState;
      },
    },
  ),
);

type PersistedSettingsPatch = Partial<
  Pick<SettingsState, "closeWindowAction">
>;

function extractValidLegacyChatWindowSettings(raw: unknown): ChatWindowSizeSetting[] {
  if (!Array.isArray(raw)) return [];
  return raw.filter((item): item is ChatWindowSizeSetting => {
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

function extractValidPersistedSettings(raw: unknown): PersistedSettingsPatch {
  if (!raw || typeof raw !== "object") return {};
  const src = raw as Record<string, unknown>;
  const patch: PersistedSettingsPatch = {};
  if (src.closeWindowAction === "ask" || src.closeWindowAction === "hide_to_tray" || src.closeWindowAction === "exit_app") patch.closeWindowAction = src.closeWindowAction;
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
        }
      } catch (err) {
        console.warn("[SettingsStore] Failed to sync storage event:", err);
      }
    }
  });
}
