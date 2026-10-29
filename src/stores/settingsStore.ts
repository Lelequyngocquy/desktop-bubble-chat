import { create } from "zustand";

interface SettingsState {
  isBubbleEnabled: boolean;
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  isAutoStart: boolean;

  setBubbleEnabled: (enabled: boolean) => void;
  toggleBubble: () => void;

  setAlwaysOnTop: (enabled: boolean) => void;
  toggleAlwaysOnTop: () => void;

  setNotificationsEnabled: (enabled: boolean) => void;
  toggleNotifications: () => void;

  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  isBubbleEnabled: false,
  isAlwaysOnTop: true,
  isNotificationsEnabled: true,
  isAutoStart: false,

  setBubbleEnabled: (enabled) => set({ isBubbleEnabled: enabled }),
  toggleBubble: () => set((state) => ({ isBubbleEnabled: !state.isBubbleEnabled })),

  setAlwaysOnTop: (enabled) => set({ isAlwaysOnTop: enabled }),
  toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),

  setNotificationsEnabled: (enabled) => set({ isNotificationsEnabled: enabled }),
  toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),

  setAutoStart: (enabled) => set({ isAutoStart: enabled }),
  toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),
}));
