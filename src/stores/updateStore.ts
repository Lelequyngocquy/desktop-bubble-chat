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

export interface UpdaterBackendPayload {
  status: string;
  version?: string;
  message?: string;
}

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
  activeUpdate: Update | null;
  setStatus: (status: UpdateStatus) => void;
  setCurrentVersion: (version: string) => void;
  setUpdateAvailable: (data: {
    availableVersion: string;
    releaseDate?: string | null;
    releaseNotes?: string | null;
    updateInstance?: Update | null;
  }) => void;
  applyBackendStatus: (payload: UpdaterBackendPayload) => void;
  setUpToDate: () => void;
  setProgress: (
    downloadedBytes: number,
    totalBytes: number,
    progressPercent: number,
    speed?: string,
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
      currentVersion: "",
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
      setStatus: (status) => set((state) => ({ status, errorMessage: status === "error" ? state.errorMessage : null })),
      setCurrentVersion: (version) => set({ currentVersion: version }),
      setUpdateAvailable: ({ availableVersion, releaseDate, releaseNotes, updateInstance = null }) =>
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
      applyBackendStatus: ({ status: s, version, message }) =>
        set((state) => {
          if (s === "downloading") {
            return {
              status: "downloading",
              availableVersion: version || state.availableVersion,
              errorMessage: null,
            };
          }
          if (s === "installing") {
            return {
              status: "installing",
              availableVersion: version || state.availableVersion,
              errorMessage: null,
            };
          }
          if (s === "error") {
            return { status: "error", errorMessage: message || "Update error" };
          }
          if (s === "up-to-date") {
            return {
              status: "up-to-date",
              availableVersion: null,
              releaseDate: null,
              releaseNotes: null,
              activeUpdate: null,
              errorMessage: null,
              progressPercent: 0,
            };
          }
          if (s === "available" && version) {
            const isBusy =
              state.status === "downloading" ||
              state.status === "downloaded" ||
              state.status === "installing";
            const keepInstance = state.availableVersion === version ? state.activeUpdate : null;
            return {
              status: isBusy ? state.status : "available",
              availableVersion: version,
              activeUpdate: keepInstance,
              errorMessage: null,
            };
          }
          return state;
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
        set((state) => {
          if (state.status !== "downloading") return state;
          return {
            downloadedBytes,
            totalBytes,
            progressPercent,
            downloadSpeed: speed || state.downloadSpeed,
          };
        }),
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
    },
  ),
);
