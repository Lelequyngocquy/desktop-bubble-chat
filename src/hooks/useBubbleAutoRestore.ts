import { useRef, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore, messengerService } from "../platforms/messenger";
import { feLog } from "../services/loggerService";
import { IPC_EVENTS, MessengerAuthStatusPayload } from "../types/ipc";

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
    } catch {
      isMainWindow = true;
    }
    if (!isMainWindow) return;

    // 2. Lắng nghe sự kiện MESSENGER_AUTH_STATUS từ Backend
    let unlisten: (() => void) | undefined;
    listen<MessengerAuthStatusPayload>(
      IPC_EVENTS.MESSENGER_AUTH_STATUS,
      async (event) => {
        const { connected, accountIndex } = event.payload;
        const idx = accountIndex || 1;

        if (connected) {
          if (restoredIndicesRef.current.has(idx)) {
            return;
          }

          const store = useMessengerStore.getState();
          const account = store.accounts.find((a) => a.index === idx);

          // Chỉ tự động khôi phục nếu trước đó tài khoản đã được người dùng bật Switch (hoặc mặc định khôi phục)
          const shouldRestore =
            account?.shouldRestoreBubble ?? account?.isBubbleVisible ?? true;

          if (shouldRestore) {
            feLog.info(
              `[useBubbleAutoRestore] Messenger ${idx} đã sẵn sàng & ổn định -> Tự động bật Switch và hiển thị Bubble mượt mà.`
            );
            restoredIndicesRef.current.add(idx);

            // Tự động bật switch trên UI Dashboard sang BẬT (SSOT)
            store.setAccountBubbleVisibility(idx, true);

            // Bật hiển thị Bubble mượt mà tại vị trí đã định sẵn mà không gây giật lag
            try {
              await messengerService.toggleBubble(true, idx);
            } catch (err) {
              feLog.error(
                `[useBubbleAutoRestore] Lỗi khi bật Bubble ${idx}:`,
                err
              );
            }
          } else {
            feLog.info(
              `[useBubbleAutoRestore] Messenger ${idx} đã sẵn sàng nhưng người dùng đang tắt Switch -> Giữ nguyên trạng thái TẮT.`
            );
            restoredIndicesRef.current.add(idx);
          }
        }
      }
    ).then((fn) => {
      unlisten = fn;
    });

    // 3. Fallback an toàn: Sau 5s nếu backend đã kết nối từ trước nhưng sự kiện bị lỡ
    const fallbackTimer = setTimeout(async () => {
      const store = useMessengerStore.getState();
      const connectedAccounts = store.accounts.filter((a) => a.isConnected);

      for (const acc of connectedAccounts) {
        if (!restoredIndicesRef.current.has(acc.index)) {
          const shouldRestore =
            acc.shouldRestoreBubble ?? acc.isBubbleVisible ?? true;
          if (shouldRestore) {
            feLog.info(
              `[useBubbleAutoRestore] [Fallback 5s] Khôi phục Bubble ${acc.index}...`
            );
            restoredIndicesRef.current.add(acc.index);
            store.setAccountBubbleVisibility(acc.index, true);
            try {
              await messengerService.toggleBubble(true, acc.index);
            } catch {}
          }
        }
      }
    }, 5000);

    return () => {
      if (unlisten) unlisten();
      clearTimeout(fallbackTimer);
    };
  }, []);
}

export default useBubbleAutoRestore;
