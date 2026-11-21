import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { useUpdateStore } from "../stores/updateStore";
import { systemService } from "./systemService";
import { notificationService } from "./notificationService";

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 MB";
  const mb = bytes / (1024 * 1024);
  if (mb < 0.1) {
    const kb = bytes / 1024;
    return `${kb.toFixed(1)} KB`;
  }
  return `${mb.toFixed(1)} MB`;
}

export function formatSpeed(bytesPerSec: number): string {
  if (!Number.isFinite(bytesPerSec) || bytesPerSec <= 0) return "0 KB/s";
  const kb = bytesPerSec / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(1)} KB/s`;
  }
  const mb = kb / 1024;
  return `${mb.toFixed(2)} MB/s`;
}

let isDownloading = false;
let cancelRequested = false;

function formatCheckTimestamp(): string {
  return new Date().toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  });
}

function mapCheckErrorMessage(err: unknown): string {
  let message = err instanceof Error ? err.message : typeof err === "string" ? err : "Unable to connect to update server.";
  if (message.includes("Could not fetch a valid release JSON") || message.includes("404")) {
    message = "New release are not available!";
  } else if (message.toLowerCase().includes("network") || message.toLowerCase().includes("failed to fetch")) {
    message = "Network error: Unable to access GitHub Releases server.";
  }
  return message;
}

export const updaterService = {
  async checkForUpdates(isSilent = false): Promise<Update | null> {
    const store = useUpdateStore.getState();
    try {
      if (!isSilent) store.setStatus("checking");
      store.setLastCheckedTime(formatCheckTimestamp());
      const update = await check();
      if (update) {
        console.log(`[AutoUpdater] Found new version: ${update.version} (Current: ${update.currentVersion})`);
        store.setUpdateAvailable({
          availableVersion: update.version,
          releaseDate: update.date,
          releaseNotes: update.body,
          updateInstance: update,
        });
        if (isSilent) {
          try {
            const granted = await notificationService.requestPermission();
            if (granted) {
              notificationService.sendNotification("DesktopBubbleChat · Bản cập nhật mới", {
                body: `Đã có phiên bản mới v${update.version}. Mở Cài đặt để cập nhật.`,
              });
            }
          } catch (notiErr) {
            console.warn("[AutoUpdater] Notification dispatch failed:", notiErr);
          }
        }
        return update;
      }
      console.log("[AutoUpdater] Application is up to date.");
      store.setUpToDate();
      return null;
    } catch (err) {
      console.error("[AutoUpdater] Error checking for updates:", err);
      if (!isSilent) store.setError(mapCheckErrorMessage(err));
      return null;
    }
  },

  async startDownload(): Promise<void> {
    if (isDownloading) return;
    const store = useUpdateStore.getState();
    let targetUpdate = store.activeUpdate;
    if (!targetUpdate) {
      targetUpdate = await this.checkForUpdates(false);
      if (!targetUpdate) return;
    }
    isDownloading = true;
    cancelRequested = false;
    store.resetProgress();
    store.setStatus("downloading");
    let total = 0;
    let downloaded = 0;
    let lastTime = Date.now();
    let lastBytes = 0;
    try {
      await targetUpdate.download((event: DownloadEvent) => {
        if (cancelRequested) {
          throw new Error("Download cancelled by user.");
        }
        if (event.event === "Started") {
          total = event.data.contentLength || 0;
          useUpdateStore.getState().setProgress(0, total, 0, "0 KB/s");
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          const now = Date.now();
          const elapsed = (now - lastTime) / 1000;
          let speedStr: string | undefined;
          if (elapsed >= 0.4) {
            const bytesDiff = downloaded - lastBytes;
            speedStr = formatSpeed(bytesDiff / elapsed);
            lastTime = now;
            lastBytes = downloaded;
          }
          const pct = total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
          useUpdateStore.getState().setProgress(downloaded, total, pct, speedStr);
        } else if (event.event === "Finished") {
          useUpdateStore.getState().setProgress(total, total, 100, "Done");
          useUpdateStore.getState().setStatus("downloaded");
        }
      });
    } catch (err) {
      const current = useUpdateStore.getState();
      if (cancelRequested) {
        console.log("[AutoUpdater] Update download cancelled.");
        current.setStatus("available");
        current.resetProgress();
      } else {
        console.error("[AutoUpdater] Error downloading update:", err);
        const msg = err instanceof Error ? err.message : "Failed to download update. Please try again.";
        current.setError(msg);
      }
    } finally {
      isDownloading = false;
      cancelRequested = false;
    }
  },

  async cancelDownload(): Promise<void> {
    cancelRequested = true;
    isDownloading = false;
    const store = useUpdateStore.getState();
    const targetUpdate = store.activeUpdate;
    store.setActiveUpdate(null);
    store.setStatus("available");
    store.resetProgress();
    if (targetUpdate) {
      try {
        await targetUpdate.close();
      } catch (e) {
        console.warn("[AutoUpdater] Close update resource:", e);
      }
    }
  },

  async installAndRelaunch(): Promise<void> {
    const store = useUpdateStore.getState();
    const targetUpdate = store.activeUpdate;
    if (!targetUpdate) {
      store.setError("No downloaded update found to install.");
      return;
    }
    try {
      store.setStatus("installing");
      await targetUpdate.install({ restartAfterInstall: true });
    } catch (err) {
      console.error("[AutoUpdater] Error installing update:", err);
      const msg = err instanceof Error ? err.message : "Failed to install update.";
      useUpdateStore.getState().setError(msg);
    }
  },

  async restartApp(): Promise<void> {
    await systemService.restartApp();
  },
};
