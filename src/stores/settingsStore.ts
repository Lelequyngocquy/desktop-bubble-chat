import { create } from "zustand";
import { persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";
import i18n from "../i18n";
import { IPC_COMMANDS } from "../types/ipc";

export type BubbleSize = "sm" | "md" | "lg";
export type AppLanguage = "vi" | "en";
export type ThemeMode = "light" | "dark" | "system";
export type CloseWindowAction = "ask" | "hide_to_tray" | "exit_app";

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
  language: AppLanguage;
  isAutoTrimEnabled: boolean;
  themeMode: ThemeMode;
  closeWindowAction: CloseWindowAction;

  setAlwaysOnTop: (enabled: boolean) => void;
  toggleAlwaysOnTop: () => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  toggleNotifications: () => void;
  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;
  setGlobalShortcutEnabled: (enabled: boolean) => void;
  toggleGlobalShortcut: () => void;
  setBubbleSize: (size: BubbleSize) => void;
  setLanguage: (lang: AppLanguage, skipIpc?: boolean) => void;
  setAutoTrimEnabled: (enabled: boolean, skipIpc?: boolean) => void;
  toggleAutoTrim: () => void;
  setThemeMode: (mode: ThemeMode, skipIpc?: boolean) => void;
  setCloseWindowAction: (action: CloseWindowAction) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      isAlwaysOnTop: true,
      isNotificationsEnabled: true,
      isAutoStart: false,
      isGlobalShortcutEnabled: true,
      bubbleSize: "md",
      language: "en",
      isAutoTrimEnabled: true,
      themeMode: "system",
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
        }
      } catch (err) {
        console.warn("[SettingsStore] Failed to sync storage event:", err);
      }
    }
  });
}
