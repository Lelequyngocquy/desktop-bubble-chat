import { create } from "zustand";
import { persist } from "zustand/middleware";

export type BubbleSize = "sm" | "md" | "lg";

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

  setAlwaysOnTop: (enabled: boolean) => void;
  toggleAlwaysOnTop: () => void;

  setNotificationsEnabled: (enabled: boolean) => void;
  toggleNotifications: () => void;

  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;

  setGlobalShortcutEnabled: (enabled: boolean) => void;
  toggleGlobalShortcut: () => void;

  setBubbleSize: (size: BubbleSize) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      isAlwaysOnTop: true,
      isNotificationsEnabled: true,
      isAutoStart: false,
      isGlobalShortcutEnabled: true,
      bubbleSize: "md",

      setAlwaysOnTop: (enabled) => set({ isAlwaysOnTop: enabled }),
      toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),

      setNotificationsEnabled: (enabled) => set({ isNotificationsEnabled: enabled }),
      toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),

      setAutoStart: (enabled) => set({ isAutoStart: enabled }),
      toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),

      setGlobalShortcutEnabled: (enabled) => set({ isGlobalShortcutEnabled: enabled }),
      toggleGlobalShortcut: () => set((state) => ({ isGlobalShortcutEnabled: !state.isGlobalShortcutEnabled })),

      setBubbleSize: (size) => set({ bubbleSize: size }),
    }),
    {
      name: "desktop_bubble_chat_settings",
    }
  )
);

// Cross-window synchronization: Khi cửa sổ main lưu cài đặt mới vào localStorage,
// các cửa sổ phụ (bubble-01 .. bubble-05) tự động cập nhật lại Zustand store đồng bộ.
if (typeof window !== "undefined") {
  window.addEventListener("storage", (event) => {
    if (event.key === "desktop_bubble_chat_settings" && event.newValue) {
      try {
        const parsed = JSON.parse(event.newValue);
        if (parsed?.state) {
          useSettingsStore.setState(parsed.state);
        }
      } catch (err) {
        console.warn("[SettingsStore] Failed to sync storage event:", err);
      }
    }
  });
}
