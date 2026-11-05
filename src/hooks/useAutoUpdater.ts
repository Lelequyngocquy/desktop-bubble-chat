import { useCallback, useEffect, useRef } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { check, type DownloadEvent } from "@tauri-apps/plugin-updater";
import { useUpdateStore } from "../stores/updateStore";
import { systemService } from "../services/systemService";

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 MB";
  const mb = bytes / (1024 * 1024);
  if (mb < 0.1) {
    const kb = bytes / 1024;
    return `${kb.toFixed(1)} KB`;
  }
  return `${mb.toFixed(1)} MB`;
}

export function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec <= 0) return "0 KB/s";
  const kb = bytesPerSec / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(1)} KB/s`;
  }
  const mb = kb / 1024;
  return `${mb.toFixed(2)} MB/s`;
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
  const activeUpdate = useUpdateStore((s) => s.activeUpdate);
  const setStatus = useUpdateStore((s) => s.setStatus);
  const setCurrentVersion = useUpdateStore((s) => s.setCurrentVersion);
  const setUpdateAvailable = useUpdateStore((s) => s.setUpdateAvailable);
  const setUpToDate = useUpdateStore((s) => s.setUpToDate);
  const setProgress = useUpdateStore((s) => s.setProgress);
  const setError = useUpdateStore((s) => s.setError);
  const setAutoCheckOnStartup = useUpdateStore((s) => s.setAutoCheckOnStartup);
  const setLastCheckedTime = useUpdateStore((s) => s.setLastCheckedTime);
  const resetProgress = useUpdateStore((s) => s.resetProgress);
  const isDownloadingRef = useRef(false);
  const cancelRequestedRef = useRef(false);

  useEffect(() => {
    getVersion()
      .then((ver) => {
        if (ver) setCurrentVersion(ver);
      })
      .catch((err) => {
        console.warn("[AutoUpdater] Failed to get app version:", err);
      });
  }, [setCurrentVersion]);

  const checkForUpdates = useCallback(
    async (isSilent = false) => {
      try {
        if (!isSilent) {
          setStatus("checking");
        }
        const nowStr = new Date().toLocaleTimeString(undefined, {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
          day: "2-digit",
          month: "2-digit",
          year: "numeric",
        });
        setLastCheckedTime(nowStr);
        const update = await check();
        if (update) {
          console.log(
            `[AutoUpdater] Found new version: ${update.version} (Current: ${update.currentVersion})`,
          );
          setUpdateAvailable({
            availableVersion: update.version,
            releaseDate: update.date,
            releaseNotes: update.body,
            updateInstance: update,
          });
        } else {
          console.log("[AutoUpdater] Application is up to date.");
          setUpToDate();
        }
      } catch (err) {
        console.error("[AutoUpdater] Error checking for updates:", err);
        let message =
          err instanceof Error
            ? err.message
            : typeof err === "string"
              ? err
              : "Unable to connect to update server.";
        if (
          message.includes("Could not fetch a valid release JSON") ||
          message.includes("404")
        ) {
          message = "New release are not available!";
        } else if (
          message.toLowerCase().includes("network") ||
          message.toLowerCase().includes("failed to fetch")
        ) {
          message = "Network error: Unable to access GitHub Releases server.";
        }
        if (!isSilent) {
          setError(message);
        }
      }
    },
    [setError, setLastCheckedTime, setStatus, setUpToDate, setUpdateAvailable],
  );

  const hasCheckedOnMountRef = useRef(false);
  useEffect(() => {
    if (autoCheckOnStartup && !hasCheckedOnMountRef.current) {
      hasCheckedOnMountRef.current = true;
      const timer = setTimeout(() => {
        checkForUpdates(true);
      }, 2500);
      return () => clearTimeout(timer);
    }
  }, [autoCheckOnStartup, checkForUpdates]);

  const startDownload = useCallback(async () => {
    if (!activeUpdate) {
      setError("No update metadata found.");
      return;
    }
    if (isDownloadingRef.current) return;
    isDownloadingRef.current = true;
    cancelRequestedRef.current = false;
    resetProgress();
    setStatus("downloading");
    let total = 0;
    let downloaded = 0;
    let lastTime = Date.now();
    let lastBytes = 0;
    try {
      await activeUpdate.download((event: DownloadEvent) => {
        if (cancelRequestedRef.current) {
          throw new Error("Download cancelled by user.");
        }
        if (event.event === "Started") {
          total = event.data.contentLength || 0;
          setProgress(0, total, 0, "0 KB/s");
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          const now = Date.now();
          const elapsed = (now - lastTime) / 1000;
          let speedStr: string | undefined = undefined;
          if (elapsed >= 0.4) {
            const bytesDiff = downloaded - lastBytes;
            const speed = bytesDiff / elapsed;
            speedStr = formatSpeed(speed);
            lastTime = now;
            lastBytes = downloaded;
          }
          const pct =
            total > 0
              ? Math.min(100, Math.round((downloaded / total) * 100))
              : 0;
          setProgress(downloaded, total, pct, speedStr);
        } else if (event.event === "Finished") {
          setProgress(total, total, 100, "Done");
          setStatus("downloaded");
        }
      });
    } catch (err) {
      if (cancelRequestedRef.current) {
        console.log("[AutoUpdater] Update download cancelled.");
        setStatus("available");
        resetProgress();
      } else {
        console.error("[AutoUpdater] Error downloading update:", err);
        const msg =
          err instanceof Error
            ? err.message
            : "Failed to download update. Please try again.";
        setError(msg);
      }
    } finally {
      isDownloadingRef.current = false;
      cancelRequestedRef.current = false;
    }
  }, [activeUpdate, resetProgress, setError, setProgress, setStatus]);

  const cancelDownload = useCallback(async () => {
    cancelRequestedRef.current = true;
    isDownloadingRef.current = false;
    if (activeUpdate) {
      try {
        await activeUpdate.close();
      } catch (e) {
        console.warn("[AutoUpdater] Close update resource:", e);
      }
    }
    setStatus("available");
    resetProgress();
  }, [activeUpdate, resetProgress, setStatus]);

  const installAndRelaunch = useCallback(async () => {
    if (!activeUpdate) {
      setError("No downloaded update found to install.");
      return;
    }
    try {
      setStatus("installing");
      await activeUpdate.install({ restartAfterInstall: true });
    } catch (err) {
      console.error("[AutoUpdater] Error installing update:", err);
      const msg =
        err instanceof Error ? err.message : "Failed to install update.";
      setError(msg);
    }
  }, [activeUpdate, setError, setStatus]);

  const restartApp = useCallback(async () => {
    await systemService.restartApp();
  }, []);

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
    hasUpdate:
      status === "available" ||
      status === "downloading" ||
      status === "downloaded",
    checkForUpdates,
    startDownload,
    cancelDownload,
    installAndRelaunch,
    restartApp,
    setAutoCheckOnStartup,
  };
}
