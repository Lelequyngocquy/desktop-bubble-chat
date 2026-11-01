import { create } from "zustand";

/**
 * Trạng thái vòng đời của Dashboard chính:
 * - initializing: App vừa bật hoặc Dashboard vừa được tái sinh sau khi đóng băng chạy ngầm.
 * - rehydrating: Đang nạp cấu hình, session đăng nhập và đồng bộ dữ liệu.
 * - ready: Toàn bộ dữ liệu cốt lõi đã nạp xong, hệ thống ổn định và sẵn sàng cho tương tác.
 */
export type DashboardLifecycleStatus = "initializing" | "rehydrating" | "ready";

interface AppLifecycleState {
  status: DashboardLifecycleStatus;
  isDashboardReady: boolean;
  statusMessage: string;

  setDashboardStatus: (status: DashboardLifecycleStatus, message?: string) => void;
  markDashboardReady: () => void;
  resetDashboardState: (status?: DashboardLifecycleStatus, message?: string) => void;
}

/**
 * Store SSOT quản lý vòng đời khởi tạo của Dashboard.
 * Lưu ý: KHÔNG dùng persist để mỗi lần cửa sổ "main" được tái tạo (sau dynamic destruction),
 * state luôn khởi đầu từ initializing để kích hoạt LoadingIndicator chặn click sớm.
 */
export const useAppLifecycleStore = create<AppLifecycleState>((set) => ({
  status: "initializing",
  isDashboardReady: false,
  statusMessage: "Đang khởi tạo không gian làm việc...",

  setDashboardStatus: (status, message) =>
    set((prev) => ({
      status,
      isDashboardReady: status === "ready",
      statusMessage: message ?? (status === "ready" ? "Đã sẵn sàng" : prev.statusMessage),
    })),

  markDashboardReady: () =>
    set({
      status: "ready",
      isDashboardReady: true,
      statusMessage: "Đã sẵn sàng",
    }),

  resetDashboardState: (status = "initializing", message = "Đang đồng bộ hóa không gian làm việc...") =>
    set({
      status,
      isDashboardReady: false,
      statusMessage: message,
    }),
}));
