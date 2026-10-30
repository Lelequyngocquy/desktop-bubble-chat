import React, { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWindow, currentMonitor } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { getAccountThemeColor, AccountThemeColor } from "../../constants/accountColors";
import { IPC_EVENTS, BubbleSpawnedPayload } from "../../types/ipc";

interface ActiveIndicator {
  id: string;
  accountIndex: number;
  y: number;
  isLeft: boolean;
  color: AccountThemeColor;
  isFadingOut: boolean;
}

/**
 * BubbleSpawnIndicator:
 * Hiển thị một mũi tên đơn sắc (không container) nằm ngang ở giữa màn hình (trục X = 50%),
 * dịch chuyển theo trục Y để khớp vị trí độ cao của Bubble (được chặn boundary bên trong app).
 * Tự động hủy hoàn toàn sau đúng 1s để giải phóng bộ nhớ.
 */
export const BubbleSpawnIndicator: React.FC = () => {
  const [indicators, setIndicators] = useState<ActiveIndicator[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const removeIndicator = useCallback((id: string) => {
    setIndicators((prev) => prev.filter((item) => item.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  const addIndicator = useCallback(
    async (accountIndex: number, bubbleX?: number, bubbleY?: number) => {
      let y = typeof window !== "undefined" ? window.innerHeight / 2 : 400;
      let isLeft = false;

      try {
        const appWin = getCurrentWindow();
        const [winPos, winSize, monitor] = await Promise.all([
          appWin.outerPosition(),
          appWin.outerSize(),
          currentMonitor(),
        ]);

        const scale = monitor?.scaleFactor || window.devicePixelRatio || 1;
        const centerX = winPos.x + winSize.width / 2;

        // Xác định hướng nằm ngang: trỏ sang trái hay phải
        if (bubbleX !== undefined) {
          isLeft = bubbleX < centerX;
        }

        // Tính vị trí Y tương đối bên trong app window và giới hạn trong boundary an toàn
        if (bubbleY !== undefined) {
          const relativeY = (bubbleY - winPos.y) / scale;
          const appHeight = window.innerHeight || winSize.height / scale;
          const topBoundary = 56; // Chừa thanh TitleBar
          const bottomBoundary = appHeight - 56; // Chừa đáy cửa sổ
          y = Math.max(topBoundary, Math.min(bottomBoundary, relativeY));
        }
      } catch {
        if (bubbleX !== undefined) {
          isLeft = bubbleX < 500;
        }
      }

      const color = getAccountThemeColor(accountIndex);
      const id = `${accountIndex}-${Date.now()}`;

      // Xóa timer cũ nếu bubble này vừa được kích hoạt
      timersRef.current.forEach((t, key) => {
        if (key.startsWith(`${accountIndex}-`)) {
          clearTimeout(t);
          timersRef.current.delete(key);
        }
      });

      // Thêm indicator
      setIndicators((prev) => [
        ...prev.filter((item) => item.accountIndex !== accountIndex),
        {
          id,
          accountIndex,
          y,
          isLeft,
          color,
          isFadingOut: false,
        },
      ]);

      // Mờ dần ở mốc 800ms
      const fadeTimer = setTimeout(() => {
        setIndicators((prev) =>
          prev.map((item) => (item.id === id ? { ...item, isFadingOut: true } : item))
        );
      }, 800);

      // Hủy hoàn toàn sau đúng 1.000ms
      const destroyTimer = setTimeout(() => {
        removeIndicator(id);
      }, 1000);

      timersRef.current.set(`${id}-fade`, fadeTimer);
      timersRef.current.set(id, destroyTimer);
    },
    [removeIndicator]
  );

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    // Chỉ kích hoạt khi chạy hàm bật bong bóng nổi (bubble-spawned)
    listen<BubbleSpawnedPayload>(IPC_EVENTS.BUBBLE_SPAWNED, (event) => {
      if (event.payload) {
        addIndicator(event.payload.accountIndex, event.payload.x, event.payload.y);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
      timersRef.current.forEach((t) => clearTimeout(t));
      timersRef.current.clear();
    };
  }, [addIndicator]);

  if (indicators.length === 0) return null;

  return (
    <>
      <style>{`
        @keyframes nudgeLeft {
          0%, 100% { transform: translateX(0); }
          50% { transform: translateX(-16px); }
        }
        @keyframes nudgeRight {
          0%, 100% { transform: translateX(0); }
          50% { transform: translateX(16px); }
        }
        .indicator-nudge-left {
          animation: nudgeLeft 0.5s ease-in-out infinite;
        }
        .indicator-nudge-right {
          animation: nudgeRight 0.5s ease-in-out infinite;
        }
      `}</style>

      <div className="fixed inset-0 pointer-events-none z-50 overflow-hidden">
        {indicators.map((item) => (
          <div
            key={item.id}
            className={`absolute transition-opacity duration-200 pointer-events-none select-none ${
              item.isFadingOut ? "opacity-0" : "opacity-100"
            }`}
            style={{
              left: "50%",
              top: `${item.y}px`,
              transform: "translate(-50%, -50%)",
            }}
          >
            {/* Chỉ duy nhất mũi tên màu nguyên bản, không container */}
            <svg
              width="64"
              height="32"
              viewBox="0 0 64 32"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              className={item.isLeft ? "indicator-nudge-left" : "indicator-nudge-right"}
              style={{
                filter: `drop-shadow(0 0 8px ${item.color.hex}) drop-shadow(0 0 16px ${item.color.hex}80)`,
              }}
            >
              {item.isLeft ? (
                /* Mũi tên nằm ngang trỏ thẳng sang mép trái */
                <path
                  d="M24 6L6 16L24 26M8 16H58"
                  stroke={item.color.hex}
                  strokeWidth="5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              ) : (
                /* Mũi tên nằm ngang trỏ thẳng sang mép phải */
                <path
                  d="M40 6L58 16L40 26M56 16H6"
                  stroke={item.color.hex}
                  strokeWidth="5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              )}
            </svg>
          </div>
        ))}
      </div>
    </>
  );
};

export default BubbleSpawnIndicator;
