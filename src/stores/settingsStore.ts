import { create } from "zustand";
import { persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";
import i18n from "../i18n";
import { IPC_COMMANDS } from "../types/ipc";

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
  setGlobalShortcutEnabled: (enabled: boolean) => void;
  toggleGlobalShortcut: () => void;
  setBubbleSize: (size: BubbleSize) => void;
  setBubblePositionMode: (mode: BubblePositionMode, skipIpc?: boolean) => void;
  setLanguage: (lang: AppLanguage, skipIpc?: boolean) => void;
  setAutoTrimEnabled: (enabled: boolean, skipIpc?: boolean) => void;
  toggleAutoTrim: () => void;
  setThemeMode: (mode: ThemeMode, skipIpc?: boolean) => void;
  setCloseWindowAction: (action: CloseWindowAction) => void;
  setUpdateMode: (mode: UpdateMode) => void;
  setChatWindowMode: (accountIndex: number, accountId: string, mode: ChatWindowSizeMode) => void | Promise<void>;
  updateChatWindowSize: (accountIndex: number, width: number, height: number) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
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
      updateMode: (() => {
        if (typeof localStorage !== "undefined") {
          try {
            const oldUpdate = localStorage.getItem("desktop_bubble_chat_update_settings");
            if (oldUpdate) {
              const parsed = JSON.parse(oldUpdate);
              if (parsed?.state?.updateMode) return parsed.state.updateMode;
            }
          } catch {}
        }
        return "auto";
      })(),
      closeWindowAction: (() => {
        if (typeof localStorage !== "undefined") {
          const val = localStorage.getItem("dbc_close_window_action");
          if (val === "hide_to_tray" || val === "exit_app") return val;
        }
        return "ask";
      })(),

      setAlwaysOnTop: (enabled) => set({ isAlwaysOnTop: enabled }),
      toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),
      setNotificationsEnabled: (enabled) => set({ isNotificationsEnabled: enabled }),
      toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),
      setAutoStart: (enabled) => set({ isAutoStart: enabled }),
      toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),
      setGlobalShortcutEnabled: (enabled) => set({ isGlobalShortcutEnabled: enabled }),
      toggleGlobalShortcut: () => set((state) => ({ isGlobalShortcutEnabled: !state.isGlobalShortcutEnabled })),
      setBubbleSize: (size) => set({ bubbleSize: size }),
      setBubblePositionMode: (mode, skipIpc = false) => {
        set({ bubblePositionMode: mode });
        if (!skipIpc) {
          invoke(IPC_COMMANDS.SET_BUBBLE_POSITION_MODE, { mode }).catch(() => {});
        }
      },
      setLanguage: (lang, skipIpc = false) => {
        set({ language: lang });
        i18n.changeLanguage(lang);
        if (!skipIpc) {
          invoke(IPC_COMMANDS.SET_APP_LANGUAGE, { language: lang }).catch(() => {});
        }
      },
      setAutoTrimEnabled: (enabled, skipIpc = false) => {
        set({ isAutoTrimEnabled: enabled });
        if (!skipIpc) {
          invoke(IPC_COMMANDS.SET_AUTO_TRIM_ENABLED, { enabled }).catch(() => {});
        }
      },
      toggleAutoTrim: () => {
        set((state) => {
          const next = !state.isAutoTrimEnabled;
          invoke(IPC_COMMANDS.SET_AUTO_TRIM_ENABLED, { enabled: next }).catch(() => {});
          return { isAutoTrimEnabled: next };
        });
      },
      setThemeMode: (mode, skipIpc = false) => {
        set({ themeMode: mode });
        applyThemeMode(mode);
        if (!skipIpc) {
          invoke(IPC_COMMANDS.SET_THEME_MODE, { themeMode: mode }).catch(() => {});
        }
      },
      setCloseWindowAction: (action) => {
        set({ closeWindowAction: action });
        if (typeof localStorage !== "undefined") {
          if (action === "ask") {
            localStorage.removeItem("dbc_close_window_action");
          } else {
            localStorage.setItem("dbc_close_window_action", action);
          }
        }
      },
      setUpdateMode: (mode) => set({ updateMode: mode }),
      setChatWindowMode: async (accountIndex, accountId, mode) => {
        set((state) => {
          const existing = state.chatWindowSettings.find((s) => s.accountIndex === accountIndex);
          if (existing) {
            return {
              chatWindowSettings: state.chatWindowSettings.map((s) =>
                s.accountIndex === accountIndex ? { ...s, mode } : s
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
        const current = useSettingsStore.getState().chatWindowSettings.find((s) => s.accountIndex === accountIndex);
        try {
          const res = await invoke<[number, number] | null>(IPC_COMMANDS.SET_CHAT_WINDOW_SIZE_CONFIG, {
            accountIndex,
            isCustom,
            width: isCustom && current?.customWidth ? current.customWidth : undefined,
            height: isCustom && current?.customHeight ? current.customHeight : undefined,
          });
          if (isCustom && res && Array.isArray(res) && res.length === 2) {
            const [w, h] = res;
            if (w >= MIN_CHAT_WIDTH && h >= MIN_CHAT_HEIGHT) {
              set((state) => ({
                chatWindowSettings: state.chatWindowSettings.map((s) =>
                  s.accountIndex === accountIndex ? { ...s, customWidth: w, customHeight: h } : s
                ),
              }));
            }
          }
        } catch {}
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
                s.accountIndex === accountIndex ? { ...s, customWidth: clampedWidth, customHeight: clampedHeight } : s
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
        invoke(IPC_COMMANDS.SET_CHAT_WINDOW_SIZE_CONFIG, {
          accountIndex,
          isCustom: true,
          width: clampedWidth,
          height: clampedHeight,
        }).catch(() => {});
      },
    }),
    {
      name: "desktop_bubble_chat_settings",
      onRehydrateStorage: () => (state) => {
        if (state?.language) {
          i18n.changeLanguage(state.language);
        }
        if (state?.themeMode) {
          applyThemeMode(state.themeMode);
        }
      },
    }
  )
);

// Cross-window synchronization
if (typeof window !== "undefined") {
  window.addEventListener("storage", (event) => {
    if (event.key === "desktop_bubble_chat_settings" && event.newValue) {
      try {
        const parsed = JSON.parse(event.newValue);
        if (parsed?.state) {
          useSettingsStore.setState(parsed.state);
          if (parsed.state.language) {
            i18n.changeLanguage(parsed.state.language);
          }
          if (parsed.state.themeMode) {
            applyThemeMode(parsed.state.themeMode);
          }
          if (parsed.state.bubblePositionMode) {
            useSettingsStore.setState({ bubblePositionMode: parsed.state.bubblePositionMode });
          }
          if (parsed.state.updateMode) {
            useSettingsStore.setState({ updateMode: parsed.state.updateMode });
          }
          if (parsed.state.chatWindowSettings) {
            useSettingsStore.setState({ chatWindowSettings: parsed.state.chatWindowSettings });
          }
        }
      } catch (err) {
        console.warn("[SettingsStore] Failed to sync storage event:", err);
      }
    }
  });
}
