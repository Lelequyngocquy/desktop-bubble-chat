import React, { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWindow, currentMonitor } from "@tauri-apps/api/window";
import { getAccountThemeColor, AccountThemeColor } from "../../constants/design";
import { useWindowStore } from "../../stores/windowStore";

interface ActiveIndicator {
  id: string;
  accountIndex: number;
  y: number;
  isLeft: boolean;
  color: AccountThemeColor;
  isFadingOut: boolean;
}

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
        if (bubbleX !== undefined) {
          isLeft = bubbleX < centerX;
        }
        if (bubbleY !== undefined) {
          const relativeY = (bubbleY - winPos.y) / scale;
          const appHeight = window.innerHeight || winSize.height / scale;
          const topBoundary = 56;
          const bottomBoundary = appHeight - 56;
          y = Math.max(topBoundary, Math.min(bottomBoundary, relativeY));
        }
      } catch {
        if (bubbleX !== undefined) {
          isLeft = bubbleX < 500;
        }
      }
      const color = getAccountThemeColor(accountIndex);
      const id = `${accountIndex}-${Date.now()}`;
      timersRef.current.forEach((t, key) => {
        if (key.startsWith(`${accountIndex}-`)) {
          clearTimeout(t);
          timersRef.current.delete(key);
        }
      });
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
      const fadeTimer = setTimeout(() => {
        setIndicators((prev) =>
          prev.map((item) => (item.id === id ? { ...item, isFadingOut: true } : item))
        );
      }, 800);
      const destroyTimer = setTimeout(() => {
        removeIndicator(id);
      }, 1000);
      timersRef.current.set(`${id}-fade`, fadeTimer);
      timersRef.current.set(id, destroyTimer);
    },
    [removeIndicator]
  );

  const lastBubbleSpawn = useWindowStore((s) => s.lastBubbleSpawn);

  useEffect(() => {
    if (lastBubbleSpawn) {
      addIndicator(lastBubbleSpawn.accountIndex, lastBubbleSpawn.x, lastBubbleSpawn.y);
    }
  }, [lastBubbleSpawn, addIndicator]);

  useEffect(() => {
    return () => {
      timersRef.current.forEach((t) => clearTimeout(t));
      timersRef.current.clear();
    };
  }, []);

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
                <path
                  d="M24 6L6 16L24 26M8 16H58"
                  stroke={item.color.hex}
                  strokeWidth="5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              ) : (
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