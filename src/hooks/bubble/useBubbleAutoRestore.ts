import { useRef, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMessengerStore } from "../../stores/messengerStore";
import { messengerService } from "../../services/messenger/messengerService";
import { feLog } from "../../services/loggerService";

const INITIAL_RESTORE_DELAY_MS = 1000; // Trì hoãn 1s sau khi Dashboard hoàn tất FCP/LCP
const STAGGER_INTERVAL_MS = 250; // Giãn cách 250ms giữa mỗi Bubble chống nghẽn Win32 thread & va chạm

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Hook chuyên trách tự động khôi phục hiển thị bong bóng nổi một cách trì hoãn & tuần tự
 * khi người dùng khởi động ứng dụng (SRP Pattern).
 *
 * Chỉ chạy 1 lần duy nhất trên cửa sổ chính ('main').
 * Khắc phục triệt để lỗi React 18 StrictMode hủy timer trong môi trường dev,
 * và đồng bộ chính xác với cờ isBubbleVisible của tài khoản.
 */
export function useBubbleAutoRestore() {
  const hasFinishedRef = useRef<boolean>(false);

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

    // Nếu đã hoàn tất khôi phục thành công trước đó thì không chạy lại
    if (hasFinishedRef.current) {
      return;
    }

    // 2. Khởi chạy luồng trì hoãn khôi phục
    const restoreTimer = setTimeout(async () => {
      if (hasFinishedRef.current) return;

      try {
        const accounts = useMessengerStore.getState().accounts;
        // Khôi phục tất cả tài khoản đang có cấu hình bật bong bóng nổi (isBubbleVisible === true)
        const candidates = accounts.filter((acc) => acc.isBubbleVisible);

        if (candidates.length === 0) {
          feLog.info("[AutoRestore] Không có tài khoản nào có cấu hình isBubbleVisible = true.");
          hasFinishedRef.current = true;
          return;
        }

        feLog.info(
          `[AutoRestore] Khởi động trình tự động khôi phục ${candidates.length} bong bóng nổi sau khởi động...`
        );
        feLog.info(
          `[AutoRestore] Danh sách tài khoản: [${candidates
            .map((c) => c.name || `Messenger ${c.index.toString().padStart(2, "0")}`)
            .join(", ")}]`
        );

        // Đánh dấu đã bắt đầu tiến trình khôi phục
        hasFinishedRef.current = true;

        // 3. Kích hoạt tuần tự từng bong bóng với khoảng cách 250ms
        for (let i = 0; i < candidates.length; i++) {
          const acc = candidates[i];
          const accName = acc.name || `Messenger ${acc.index.toString().padStart(2, "0")}`;

          feLog.info(
            `[AutoRestore] [${i + 1}/${candidates.length}] Đang hiển thị Bubble cho ${accName} (index: ${acc.index})...`
          );

          try {
            await messengerService.toggleBubble(true, acc.index);
          } catch (err) {
            feLog.error(
              `[AutoRestore] Lỗi khi kích hoạt Bubble cho ${accName}:`,
              err
            );
          }

          // Giãn cách 250ms giữa các Bubble để Rust tính toán tọa độ xếp chồng chống va chạm
          if (i + 1 < candidates.length) {
            await sleep(STAGGER_INTERVAL_MS);
          }
        }

        feLog.info("[AutoRestore] Hoàn tất khôi phục toàn bộ bong bóng thành công!");
      } catch (globalErr) {
        feLog.error("[AutoRestore] Lỗi ngoại lệ trong quá trình auto-restore:", globalErr);
      }
    }, INITIAL_RESTORE_DELAY_MS);

    return () => {
      // Trong React 18 Strict Mode, nếu unmount trước khi timer chạy, clearTimeout sẽ hủy timer cũ
      // Nhưng hasFinishedRef.current vẫn là false, nên lần remount tiếp theo sẽ hẹn giờ lại chính xác!
      clearTimeout(restoreTimer);
    };
  }, []);
}

export default useBubbleAutoRestore;
