import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppLifecycleStore } from "../stores/appLifecycleStore";
import { useSystemUserStore } from "../stores/systemUserStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useMessengerStore } from "../platforms/messenger/stores/messengerStore";
import { systemService } from "../services/systemService";
import { messengerService } from "../platforms/messenger/services/messengerService";
import { feLog } from "../services/loggerService";

/**
 * Hook điều phối quá trình bootstrap và khởi tạo dữ liệu cho Dashboard chính:
 * 1. Nạp System Username vào systemUserStore.
 * 2. Đồng bộ trạng thái Autostart với Registry Windows.
 * 3. Kiểm tra tính hợp lệ của các phiên đăng nhập Messenger.
 * 4. Chờ settling buffer (200ms) để DOM & Zustand state hoàn toàn ổn định.
 * 5. Chuyển dashboardStatus sang "ready" và mở khóa tương tác (LoadingIndicator fade-out).
 *
 * Chỉ chạy trên cửa sổ Dashboard chính ('main').
 */
export function useDashboardBootstrap() {
  const isBootstrappingRef = useRef(false);
  const setDashboardStatus = useAppLifecycleStore((s) => s.setDashboardStatus);
  const markDashboardReady = useAppLifecycleStore((s) => s.markDashboardReady);
  const isDashboardReady = useAppLifecycleStore((s) => s.isDashboardReady);

  useEffect(() => {
    let isMainWindow = true;
    try {
      const label = getCurrentWindow()?.label;
      if (label && label !== "main") {
        isMainWindow = false;
      }
    } catch {
      isMainWindow = true;
    }
    if (!isMainWindow) return;

    if (isBootstrappingRef.current) return;
    isBootstrappingRef.current = true;

    feLog.info("[useDashboardBootstrap] Bắt đầu quá trình bootstrap Dashboard...");
    setDashboardStatus("initializing", "Đang khởi tạo không gian làm việc...");

    // Safety timeout: Tối đa 3.5s nếu một IPC call bị trễ mạng/backend
    const safetyTimer = setTimeout(() => {
      if (!useAppLifecycleStore.getState().isDashboardReady) {
        feLog.warn("[useDashboardBootstrap] [Safety 3.5s] Buộc mở khóa Dashboard do quá thời gian bootstrap.");
        markDashboardReady();
      }
    }, 3500);

    const runBootstrap = async () => {
      try {
        // 1. Nạp tên người dùng hệ điều hành
        setDashboardStatus("rehydrating", "Đang nạp hồ sơ người dùng...");
        try {
          const username = await systemService.getSystemUsername();
          useSystemUserStore.getState().setSystemUser(username);
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Không thể lấy system username:", err);
        }

        // 2. Đồng bộ trạng thái Autostart từ Windows Registry
        try {
          const autostart = await systemService.isAutostartEnabled();
          useSettingsStore.getState().setAutoStart(autostart);
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Không thể kiểm tra autostart:", err);
        }

        // 2b. Đồng bộ cấu hình người dùng (Bubble Size, Always on Top) từ Backend SSOT Cache
        try {
          const userSettings = await systemService.getUserSettings();
          if (userSettings) {
            if (
              userSettings.bubble_size &&
              (userSettings.bubble_size === "sm" ||
                userSettings.bubble_size === "md" ||
                userSettings.bubble_size === "lg")
            ) {
              useSettingsStore.getState().setBubbleSize(userSettings.bubble_size);
            }
            if (typeof userSettings.is_always_on_top === "boolean") {
              useSettingsStore.getState().setAlwaysOnTop(userSettings.is_always_on_top);
            }
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Không thể đồng bộ user settings từ cache:", err);
        }

        // 3. Đồng bộ trạng thái tài khoản Messenger
        setDashboardStatus("rehydrating", "Đang đồng bộ kết nối tài khoản...");
        try {
          const accounts = useMessengerStore.getState().accounts;
          for (const acc of accounts) {
            try {
              const res = await messengerService.checkAuth(acc.index);
              if (res && res.connected) {
                useMessengerStore.getState().updateAccount(acc.id, {
                  isConnected: true,
                });
              }
            } catch {
              // Bỏ qua lỗi kết nối từng tài khoản, không làm dừng bootstrap
            }
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Không thể đồng bộ tài khoản Messenger:", err);
        }

        // 4. Đồng bộ snapshot thực tế các Bong bóng đang hiển thị từ Backend (SSOT - Read Only)
        try {
          const activeIndices = await messengerService.getActiveBubbleAccounts();
          const activeSet = new Set(activeIndices);
          const store = useMessengerStore.getState();
          for (const acc of store.accounts) {
            const isLive = activeSet.has(acc.index);
            if (acc.isBubbleVisible !== isLive) {
              store.setAccountBubbleVisibility(acc.index, isLive);
            }
          }
        } catch (err) {
          feLog.warn("[useDashboardBootstrap] Không thể lấy snapshot active bubbles:", err);
        }

        // 5. Settling buffer 200ms giúp React layout ổn định không bị giật nhấp nháy
        await new Promise((resolve) => setTimeout(resolve, 200));

        feLog.info("[useDashboardBootstrap] Hoàn tất bootstrap! Dashboard đã sẵn sàng.");
        markDashboardReady();
      } catch (err) {
        feLog.error("[useDashboardBootstrap] Lỗi không xác định trong quá trình bootstrap:", err);
        markDashboardReady();
      } finally {
        clearTimeout(safetyTimer);
      }
    };

    runBootstrap();

    return () => {
      clearTimeout(safetyTimer);
    };
  }, [setDashboardStatus, markDashboardReady]);

  return { isDashboardReady };
}

export default useDashboardBootstrap;
