import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Update } from "@tauri-apps/plugin-updater";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "up-to-date"
  | "available"
  | "downloading"
  | "downloaded"
  | "installing"
  | "error";

export interface UpdateState {
  status: UpdateStatus;
  currentVersion: string;
  availableVersion: string | null;
  releaseDate: string | null;
  releaseNotes: string | null;
  downloadedBytes: number;
  totalBytes: number;
  progressPercent: number;
  downloadSpeed: string;
  errorMessage: string | null;
  lastCheckedTime: string | null;
  autoCheckOnStartup: boolean;

  // Active Update instance reference from Tauri plugin
  activeUpdate: Update | null;

  // Actions
  setStatus: (status: UpdateStatus) => void;
  setCurrentVersion: (version: string) => void;
  setUpdateAvailable: (data: {
    availableVersion: string;
    releaseDate?: string;
    releaseNotes?: string;
    updateInstance: Update;
  }) => void;
  setUpToDate: () => void;
  setProgress: (
    downloadedBytes: number,
    totalBytes: number,
    progressPercent: number,
    speed?: string
  ) => void;
  setError: (error: string) => void;
  setAutoCheckOnStartup: (enabled: boolean) => void;
  setLastCheckedTime: (time: string) => void;
  setActiveUpdate: (update: Update | null) => void;
  resetProgress: () => void;
}

export const useUpdateStore = create<UpdateState>()(
  persist(
    (set) => ({
      status: "idle",
      currentVersion: "0.1.0",
      availableVersion: null,
      releaseDate: null,
      releaseNotes: null,
      downloadedBytes: 0,
      totalBytes: 0,
      progressPercent: 0,
      downloadSpeed: "0 KB/s",
      errorMessage: null,
      lastCheckedTime: null,
      autoCheckOnStartup: true,
      activeUpdate: null,

      setStatus: (status) => set({ status, errorMessage: status === "error" ? undefined : null }),
      setCurrentVersion: (version) => set({ currentVersion: version }),
      setUpdateAvailable: ({
        availableVersion,
        releaseDate,
        releaseNotes,
        updateInstance,
      }) =>
        set({
          status: "available",
          availableVersion,
          releaseDate: releaseDate || null,
          releaseNotes: releaseNotes || null,
          activeUpdate: updateInstance,
          errorMessage: null,
          progressPercent: 0,
          downloadedBytes: 0,
          totalBytes: 0,
        }),
      setUpToDate: () =>
        set({
          status: "up-to-date",
          availableVersion: null,
          releaseDate: null,
          releaseNotes: null,
          activeUpdate: null,
          errorMessage: null,
          progressPercent: 0,
        }),
      setProgress: (downloadedBytes, totalBytes, progressPercent, speed) =>
        set((state) => ({
          status: "downloading",
          downloadedBytes,
          totalBytes,
          progressPercent,
          downloadSpeed: speed || state.downloadSpeed,
        })),
      setError: (error) => set({ status: "error", errorMessage: error }),
      setAutoCheckOnStartup: (enabled) => set({ autoCheckOnStartup: enabled }),
      setLastCheckedTime: (time) => set({ lastCheckedTime: time }),
      setActiveUpdate: (update) => set({ activeUpdate: update }),
      resetProgress: () =>
        set({
          downloadedBytes: 0,
          totalBytes: 0,
          progressPercent: 0,
          downloadSpeed: "0 KB/s",
        }),
    }),
    {
      name: "desktop_bubble_chat_update_settings",
      partialize: (state) => ({
        autoCheckOnStartup: state.autoCheckOnStartup,
        lastCheckedTime: state.lastCheckedTime,
      }),
    }
  )
);
