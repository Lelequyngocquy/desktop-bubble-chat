import { useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useUpdateStore, type UpdaterBackendPayload } from "../stores/updateStore";
import { useSettingsStore } from "../stores/settingsStore";
import { updaterService, formatBytes, formatSpeed } from "../services/updaterService";
import { safeListen } from "../utils/tauriEvents";
import { IPC_EVENTS } from "../types/ipc";

export { formatBytes, formatSpeed };

let hasInitiatedStartupCheck = false;

/**
 * Mounts once at application root to synchronize backend updater events and trigger startup check.
 */
export function useAutoUpdaterLifecycle() {
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    getVersion()
      .then((ver) => {
        if (!disposed && ver) useUpdateStore.getState().setCurrentVersion(ver);
      })
      .catch((err) => {
        console.warn("[AutoUpdater] Failed to get app version:", err);
      });
    safeListen<UpdaterBackendPayload>(
      IPC_EVENTS.UPDATER_STATUS,
      (event) => {
        if (!disposed) {
          useUpdateStore.getState().applyBackendStatus(event.payload);
        }
      },
      () => disposed,
      (fn) => {
        unlisten = fn;
      },
    ).catch((err) => {
      console.warn("[AutoUpdater] Failed to attach updater-status listener:", err);
    });
    const timer = !hasInitiatedStartupCheck
      ? setTimeout(() => {
          if (disposed || hasInitiatedStartupCheck) return;
          hasInitiatedStartupCheck = true;
          const currentMode = useSettingsStore.getState().updateMode;
          if (currentMode === "manual" && useUpdateStore.getState().autoCheckOnStartup) {
            updaterService.checkForUpdates(true).catch(() => {});
          }
        }, 3000)
      : undefined;
    return () => {
      disposed = true;
      if (timer) clearTimeout(timer);
      unlisten?.();
    };
  }, []);
}

export function useAutoUpdater() {
  const status = useUpdateStore((s) => s.status);
  const currentVersion = useUpdateStore((s) => s.currentVersion);
  const availableVersion = useUpdateStore((s) => s.availableVersion);
  const releaseDate = useUpdateStore((s) => s.releaseDate);
  const releaseNotes = useUpdateStore((s) => s.releaseNotes);
  const downloadedBytes = useUpdateStore((s) => s.downloadedBytes);
  const totalBytes = useUpdateStore((s) => s.totalBytes);
  const progressPercent = useUpdateStore((s) => s.progressPercent);
  const downloadSpeed = useUpdateStore((s) => s.downloadSpeed);
  const errorMessage = useUpdateStore((s) => s.errorMessage);
  const lastCheckedTime = useUpdateStore((s) => s.lastCheckedTime);
  const autoCheckOnStartup = useUpdateStore((s) => s.autoCheckOnStartup);
  const setAutoCheckOnStartup = useUpdateStore((s) => s.setAutoCheckOnStartup);
  const updateMode = useSettingsStore((s) => s.updateMode);
  const setUpdateMode = useSettingsStore((s) => s.setUpdateMode);
  return {
    status,
    currentVersion,
    availableVersion,
    releaseDate,
    releaseNotes,
    downloadedBytes,
    totalBytes,
    progressPercent,
    downloadSpeed,
    errorMessage,
    lastCheckedTime,
    autoCheckOnStartup,
    updateMode,
    hasUpdate:
      status === "available" ||
      status === "downloading" ||
      status === "downloaded" ||
      status === "installing",
    checkForUpdates: updaterService.checkForUpdates.bind(updaterService),
    startDownload: updaterService.startDownload.bind(updaterService),
    cancelDownload: updaterService.cancelDownload.bind(updaterService),
    installAndRelaunch: updaterService.installAndRelaunch.bind(updaterService),
    restartApp: updaterService.restartApp.bind(updaterService),
    setAutoCheckOnStartup,
    setUpdateMode,
  };
}
