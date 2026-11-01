import React, { useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { useAppLifecycleStore } from "../../stores/appLifecycleStore";

/**
 * DashboardLoadingOverlay:
 * Lớp phủ khóa tương tác (interaction blocker) xuất hiện khi:
 * 1. App vừa khởi động (Cold Launch) chưa nạp xong dữ liệu nền.
 * 2. Dashboard vừa được tái sinh từ ngủ đông (Re-open from Tray / Dynamic Destruction).
 *
 * Chức năng:
 * - Hiển thị spinner xoay vòng mượt mà và thông điệp trạng thái.
 * - Khóa hoàn toàn click chuột trên Workspace nhằm chống nghẽn IPC & race conditions.
 * - Khi isDashboardReady = true, overlay mờ dần mượt mà và tự động mở khóa tương tác.
 * - Tích hợp Fallback Timer (4.0s) bảo đảm không bao giờ làm treo UI người dùng.
 */
export const DashboardLoadingOverlay: React.FC = () => {
  const isDashboardReady = useAppLifecycleStore((s) => s.isDashboardReady);
  const statusMessage = useAppLifecycleStore((s) => s.statusMessage);
  const markDashboardReady = useAppLifecycleStore((s) => s.markDashboardReady);

  // Trạng thái giữ DOM trong lúc hiệu ứng mờ dần (300ms) kết thúc
  const [shouldRender, setShouldRender] = useState(!isDashboardReady);

  useEffect(() => {
    if (isDashboardReady) {
      const timer = setTimeout(() => {
        setShouldRender(false);
      }, 350);
      return () => clearTimeout(timer);
    } else {
      setShouldRender(true);
      // Fallback timer an toàn: Tối đa 4 giây nếu vì lý do nào đó IPC bị trễ, tự động mở khóa UI
      const safetyTimer = setTimeout(() => {
        markDashboardReady();
      }, 4000);
      return () => clearTimeout(safetyTimer);
    }
  }, [isDashboardReady, markDashboardReady]);

  if (!shouldRender) return null;

  return (
    <div
      className={`absolute inset-0 z-50 flex items-center justify-center bg-[#EEF2F9]/85 backdrop-blur-[3px] select-none transition-opacity duration-300 ease-out ${
        isDashboardReady ? "opacity-0 pointer-events-none" : "opacity-100 pointer-events-auto"
      }`}
      aria-live="polite"
      aria-busy={!isDashboardReady}
    >
      <div className="bg-white/95 border border-[#E2E7EF] shadow-window rounded-2xl px-6 py-4 flex items-center gap-4 max-w-sm">
        <div className="relative flex items-center justify-center">
          <Loader2 className="w-6 h-6 animate-spin text-[#0A78FF]" />
        </div>
        <div className="flex flex-col min-w-0">
          <span className="text-sm font-semibold text-[#202733] truncate">
            {statusMessage || "Đang khởi tạo không gian làm việc..."}
          </span>
          <span className="text-xs text-[#6B7A90] mt-0.5">
            Đang ổn định hệ sinh thái & kết nối
          </span>
        </div>
      </div>
    </div>
  );
};

export default DashboardLoadingOverlay;
