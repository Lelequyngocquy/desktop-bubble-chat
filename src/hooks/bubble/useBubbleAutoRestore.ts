import { useRef, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore } from "../../stores/messengerStore";
import { messengerService } from "../../services/messenger/messengerService";
import { feLog } from "../../services/loggerService";
import { IPC_EVENTS, MessengerAuthStatusPayload } from "../../types/ipc";

/**
 * Hook chuyên trách tự động khôi phục hiển thị bong bóng nổi SAU KHI Messenger load xong và đạt trạng thái ổn định.
 *
 * Tiêu chí kỹ thuật:
 * 1. Khi App vừa bật: Nút switch ở trạng thái TẮT, Bubble KHÔNG hiển thị ngay (loại bỏ hoàn toàn rung giật & nhảy vị trí).
 * 2. Webview Messenger được nạp ngầm trong nền.
 * 3. Khi Messenger báo cáo hoàn tất nạp & xác thực ổn định (MESSENGER_AUTH_STATUS: connected === true):
 *    - Tự động bật nút Switch trên UI Dashboard sang BẬT.
 *    - Hiển thị Bubble tương ứng mượt mà ở vị trí đã định sẵn.
 *
 * Chỉ chạy 1 lần duy nhất trên cửa sổ chính ('main').
 */
export function useBubbleAutoRestore() {
  const restoredIndicesRef = useRef<Set<number>>(new Set());

  useEffect(() => {
    // 1. Chỉ thực thi trên cửa sổ Dashboard chính ('main'), tuyệt đối không chạy trên cửa sổ bubble
    let isMainWindow = true;
    try {
      const label = getCurrentWindow()?.label;
      if (label && label !== "main") {
        isMainWindow = false;
      }
    } catch {}

    if (
      typeof window !== "undefined" &&
      (window.location.search.includes("window=bubble") || window.location.hash.includes("bubble"))
    ) {
      isMainWindow = false;
    }

    if (!isMainWindow) {
      return;
    }

    const restoreAccountIfEligible = async (accountIndex: number) => {
      if (restoredIndicesRef.current.has(accountIndex)) return;

      const accounts = useMessengerStore.getState().accounts;
      const acc = accounts.find((a) => a.index === accountIndex);
      if (!acc) return;

      // Chỉ khôi phục nếu tài khoản được đánh dấu shouldRestoreBubble (người dùng trước đó đã bật)
      if (!acc.shouldRestoreBubble) {
        return;
      }

      // Đánh dấu tài khoản này đã được khôi phục để tránh gọi lặp
      restoredIndicesRef.current.add(accountIndex);

      const accName = acc.name || `Messenger ${acc.index.toString().padStart(2, "0")}`;
      feLog.info(
        `[AutoRestore] ${accName} (index: ${accountIndex}) đã load xong & ổn định. Bật Switch và hiển thị Bubble!`
      );

      // 1. Chuyển trạng thái Switch sang BẬT trên UI Dashboard
      useMessengerStore.getState().setAccountBubbleVisibility(accountIndex, true);

      // 2. Kích hoạt hiển thị Bong bóng nổi
      try {
        await messengerService.toggleBubble(true, accountIndex);
      } catch (err) {
        feLog.error(`[AutoRestore] Lỗi khi hiển thị Bubble cho ${accName}:`, err);
      }
    };

    // 2. Lắng nghe sự kiện Messenger nạp xong và xác thực ổn định từ backend
    let unlistenAuth: (() => void) | undefined;
    listen<MessengerAuthStatusPayload>(IPC_EVENTS.MESSENGER_AUTH_STATUS, async (event) => {
      if (event.payload?.connected) {
        const idx = event.payload.accountIndex || 1;
        await restoreAccountIfEligible(idx);
      }
    }).then((un) => {
      unlistenAuth = un;
    });

    // 3. Kiểm tra các tài khoản đã sẵn sàng từ trước (health-check)
    const accounts = useMessengerStore.getState().accounts;
    for (const acc of accounts) {
      if (acc.shouldRestoreBubble) {
        messengerService
          .checkAuth(acc.index)
          .then(async (res) => {
            if (res && res.connected) {
              await restoreAccountIfEligible(acc.index);
            }
          })
          .catch(() => {});
      }
    }

    return () => {
      if (unlistenAuth) unlistenAuth();
    };
  }, []);
}

export default useBubbleAutoRestore;
