import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMessengerStore, messengerService } from "../platforms/messenger";
import { useAppLifecycleStore } from "../stores/appLifecycleStore";
import { feLog } from "../services/loggerService";

/**
 * Hook chuyên trách đồng bộ trạng thái hiển thị bong bóng nổi theo thời gian thực (SSOT):
 * 1. Tuân thủ nguyên tắc: Dashboard chỉ là nơi lắng nghe (Listener) & truy vấn Snapshot (Read-Only).
 * 2. Tuyệt đối KHÔNG tự ý phát lệnh `toggleBubble(true)` làm đột biến trạng thái của Backend khi init.
 * 3. Khi Dashboard Ready: Truy vấn `getActiveBubbleAccounts()` từ Backend và cập nhật switch trên UI khớp 1:1.
 *
 * Chỉ chạy 1 lần trên cửa sổ chính ('main').
 */
export function useBubbleAutoRestore() {
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

    if (!isDashboardReady) return;

    // Khi Dashboard chuyển sang Ready: Đồng bộ snapshot thực tế từ Backend để hiển thị switch UI
    messengerService
      .getActiveBubbleAccounts()
      .then((activeIndices) => {
        const activeSet = new Set(activeIndices);
        const store = useMessengerStore.getState();
        for (const acc of store.accounts) {
          const isLive = activeSet.has(acc.index);
          if (acc.isBubbleVisible !== isLive) {
            feLog.info(
              `[useBubbleAutoRestore] Đồng bộ trạng thái Bubble ${acc.index} theo Backend SSOT: ${isLive ? "BẬT" : "TẮT"}`
            );
            store.setAccountBubbleVisibility(acc.index, isLive);
          }
        }
      })
      .catch((err) => {
        feLog.warn("[useBubbleAutoRestore] Lỗi khi truy vấn active bubble snapshot:", err);
      });
  }, [isDashboardReady]);
}

export default useBubbleAutoRestore;
