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

  // 1. Đồng bộ phiên bản ứng dụng thực tế từ Tauri App metadata
  useEffect(() => {
    getVersion()
      .then((ver) => {
        if (ver) setCurrentVersion(ver);
      })
      .catch((err) => {
        console.warn("[AutoUpdater] Không thể lấy version:", err);
      });
  }, [setCurrentVersion]);

  // 2. Hàm kiểm tra bản cập nhật
  const checkForUpdates = useCallback(
    async (isSilent = false) => {
      try {
        if (!isSilent) {
          setStatus("checking");
        }

        const nowStr = new Date().toLocaleTimeString("vi-VN", {
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
            `[AutoUpdater] Tìm thấy phiên bản mới: ${update.version} (Hiện tại: ${update.currentVersion})`,
          );
          setUpdateAvailable({
            availableVersion: update.version,
            releaseDate: update.date,
            releaseNotes: update.body,
            updateInstance: update,
          });
        } else {
          console.log("[AutoUpdater] Ứng dụng đã ở phiên bản mới nhất.");
          setUpToDate();
        }
      } catch (err) {
        console.error("[AutoUpdater] Lỗi khi kiểm tra cập nhật:", err);
        let message =
          err instanceof Error
            ? err.message
            : typeof err === "string"
              ? err
              : "Không thể kết nối đến máy chủ cập nhật.";

        if (
          message.includes("Could not fetch a valid release JSON") ||
          message.includes("404")
        ) {
          message = "Chưa có bản phát hành mới!";
        } else if (
          message.toLowerCase().includes("network") ||
          message.toLowerCase().includes("failed to fetch")
        ) {
          message =
            "Lỗi kết nối mạng: Không thể truy cập máy chủ GitHub Releases.";
        }

        if (!isSilent) {
          setError(message);
        }
      }
    },
    [setError, setLastCheckedTime, setStatus, setUpToDate, setUpdateAvailable],
  );

  // 3. Tự động kiểm tra khi khởi động (Silent check 1 lần)
  const hasCheckedOnMountRef = useRef(false);
  useEffect(() => {
    if (autoCheckOnStartup && !hasCheckedOnMountRef.current) {
      hasCheckedOnMountRef.current = true;
      // Delay nhẹ 2.5s để nhường tài nguyên cho quá trình khởi động UI
      const timer = setTimeout(() => {
        checkForUpdates(true);
      }, 2500);
      return () => clearTimeout(timer);
    }
  }, [autoCheckOnStartup, checkForUpdates]);

  // 4. Bắt đầu tải bản cập nhật
  const startDownload = useCallback(async () => {
    if (!activeUpdate) {
      setError("Không tìm thấy thông tin bản cập nhật.");
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
          throw new Error("Người dùng đã hủy quá trình tải.");
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
          setProgress(total, total, 100, "Hoàn tất");
          setStatus("downloaded");
        }
      });
    } catch (err) {
      if (cancelRequestedRef.current) {
        console.log("[AutoUpdater] Đã hủy tải bản cập nhật.");
        setStatus("available");
        resetProgress();
      } else {
        console.error("[AutoUpdater] Lỗi khi tải bản cập nhật:", err);
        const msg =
          err instanceof Error
            ? err.message
            : "Tải bản cập nhật thất bại. Vui lòng thử lại.";
        setError(msg);
      }
    } finally {
      isDownloadingRef.current = false;
      cancelRequestedRef.current = false;
    }
  }, [activeUpdate, resetProgress, setError, setProgress, setStatus]);

  // 5. Dừng / Hủy quá trình tải
  const cancelDownload = useCallback(async () => {
    cancelRequestedRef.current = true;
    isDownloadingRef.current = false;
    if (activeUpdate) {
      try {
        await activeUpdate.close();
      } catch (e) {
        console.warn("[AutoUpdater] Đóng update resource:", e);
      }
    }
    setStatus("available");
    resetProgress();
  }, [activeUpdate, resetProgress, setStatus]);

  // 6. Cài đặt và kích hoạt bản mới
  const installAndRelaunch = useCallback(async () => {
    if (!activeUpdate) {
      setError("Không có bản cập nhật nào đã tải để cài đặt.");
      return;
    }

    try {
      setStatus("installing");
      // Trên Windows, install() kích hoạt bộ cài và tự động thoát / khởi chạy lại app mới
      await activeUpdate.install({ restartAfterInstall: true });
    } catch (err) {
      console.error("[AutoUpdater] Lỗi khi cài đặt bản cập nhật:", err);
      const msg =
        err instanceof Error ? err.message : "Cài đặt bản cập nhật thất bại.";
      setError(msg);
    }
  }, [activeUpdate, setError, setStatus]);

  // 7. Khởi động lại ứng dụng thủ công (Reset app)
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
