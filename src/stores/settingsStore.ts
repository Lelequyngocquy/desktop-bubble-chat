import { create } from "zustand";

interface SettingsState {
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  isAutoStart: boolean;

  setAlwaysOnTop: (enabled: boolean) => void;
  toggleAlwaysOnTop: () => void;

  setNotificationsEnabled: (enabled: boolean) => void;
  toggleNotifications: () => void;

  setAutoStart: (enabled: boolean) => void;
  toggleAutoStart: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  isAlwaysOnTop: true,
  isNotificationsEnabled: true,
  isAutoStart: false,

  setAlwaysOnTop: (enabled) => set({ isAlwaysOnTop: enabled }),
  toggleAlwaysOnTop: () => set((state) => ({ isAlwaysOnTop: !state.isAlwaysOnTop })),

  setNotificationsEnabled: (enabled) => set({ isNotificationsEnabled: enabled }),
  toggleNotifications: () => set((state) => ({ isNotificationsEnabled: !state.isNotificationsEnabled })),

  setAutoStart: (enabled) => set({ isAutoStart: enabled }),
  toggleAutoStart: () => set((state) => ({ isAutoStart: !state.isAutoStart })),
}));
