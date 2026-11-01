import { useRef, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMessengerStore, messengerService } from "../platforms/messenger";
import { useAppLifecycleStore } from "../stores/appLifecycleStore";
import { feLog } from "../services/loggerService";
import { IPC_EVENTS, MessengerAuthStatusPayload } from "../types/ipc";

/**
 * Hook chuyên trách tự động khôi phục hiển thị bong bóng nổi SAU KHI Dashboard và Messenger load xong và đạt trạng thái ổn định.
 *
 * Tiêu chí kỹ thuật:
 * 1. Khi App vừa bật (Cold Launch): Toàn bộ Bubble mặc định ĐÓNG.
 * 2. Dashboard hoàn tất nạp dữ liệu nền và đạt isDashboardReady = true.
 * 3. Khi Messenger báo cáo hoàn tất nạp & xác thực ổn định (MESSENGER_AUTH_STATUS: connected === true):
 *    - Tự động bật nút Switch trên UI Dashboard sang BẬT.
 *    - Hiển thị Bubble tương ứng mượt mà ở vị trí đã định sẵn.
 *
 * Chỉ chạy 1 lần duy nhất trên cửa sổ chính ('main').
 */
export function useBubbleAutoRestore() {
  const restoredIndicesRef = useRef<Set<number>>(new Set());
  const isDashboardReady = useAppLifecycleStore((s) => s.isDashboardReady);

  // 1. Lắng nghe sự kiện MESSENGER_AUTH_STATUS từ Backend
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

          // Kiểm tra cổng chặn: Nếu Dashboard chưa Ready, tạm hoãn để effect [isDashboardReady] xử lý
          const isReady = useAppLifecycleStore.getState().isDashboardReady;
          if (!isReady) {
            feLog.info(
              `[useBubbleAutoRestore] Messenger ${idx} báo kết nối nhưng Dashboard chưa Ready -> Tạm hoãn mở Bubble cho đến khi Dashboard ổn định.`
            );
            return;
          }

          const store = useMessengerStore.getState();
          const account = store.accounts.find((a) => a.index === idx);

          const shouldRestore =
            account?.shouldRestoreBubble ?? account?.isBubbleVisible ?? true;

          if (shouldRestore) {
            feLog.info(
              `[useBubbleAutoRestore] Messenger ${idx} đã sẵn sàng & Dashboard Ready -> Tự động bật Switch và hiển thị Bubble.`
            );
            restoredIndicesRef.current.add(idx);

            store.setAccountBubbleVisibility(idx, true);

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

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // 2. Kích hoạt khôi phục Bubble khi Dashboard chuyển sang trạng thái Ready (Ổn định 100%)
  useEffect(() => {
    if (!isDashboardReady) return;

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

    const restorePendingBubbles = async () => {
      const store = useMessengerStore.getState();
      const connectedAccounts = store.accounts.filter((a) => a.isConnected);

      for (const acc of connectedAccounts) {
        if (!restoredIndicesRef.current.has(acc.index)) {
          const shouldRestore =
            acc.shouldRestoreBubble ?? acc.isBubbleVisible ?? true;
          if (shouldRestore) {
            feLog.info(
              `[useBubbleAutoRestore] [Dashboard Ready] Tự động khôi phục Bubble ${acc.index} (${acc.name})...`
            );
            restoredIndicesRef.current.add(acc.index);
            store.setAccountBubbleVisibility(acc.index, true);
            try {
              await messengerService.toggleBubble(true, acc.index);
            } catch (err) {
              feLog.error(
                `[useBubbleAutoRestore] Lỗi khi bật Bubble ${acc.index}:`,
                err
              );
            }
          }
        }
      }
    };

    restorePendingBubbles();
  }, [isDashboardReady]);

  // 3. Fallback an toàn 5s nếu sự kiện auth bị trễ
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

    const fallbackTimer = setTimeout(async () => {
      if (!useAppLifecycleStore.getState().isDashboardReady) return;

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
      clearTimeout(fallbackTimer);
    };
  }, []);
}

export default useBubbleAutoRestore;
