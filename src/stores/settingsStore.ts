import { create } from "zustand";
import { persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";
import i18n from "../i18n";
import { IPC_COMMANDS } from "../types/ipc";

export type BubbleSize = "sm" | "md" | "lg";
export type AppLanguage = "vi" | "en";

export const BUBBLE_SIZE_PIXELS: Record<BubbleSize, number> = {
  sm: 48,
  md: 56,
  lg: 64,
};

interface SettingsState {
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  isAutoStart: boolean;
  isGlobalShortcutEnabled: boolean;
  bubbleSize: BubbleSize;
  language: AppLanguage;
  isAutoTrimEnabled: boolean;

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
    }),
    {
      name: "desktop_bubble_chat_settings",
      onRehydrateStorage: () => (state) => {
        if (state?.language) {
          i18n.changeLanguage(state.language);
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
        }
      } catch (err) {
        console.warn("[SettingsStore] Failed to sync storage event:", err);
      }
    }
  });
}
