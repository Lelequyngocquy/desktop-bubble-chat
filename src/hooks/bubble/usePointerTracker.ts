import { useRef, useState, useCallback, useEffect } from "react";
import { getCurrentWindow, currentMonitor } from "@tauri-apps/api/window";

export interface PointerDragState {
  isDown: boolean;
  hasMoved: boolean;
  startScreenX: number;
  startScreenY: number;
  initialWindowX: number;
  initialWindowY: number;
  bottomEdge: number;
  scale: number;
}

interface UsePointerTrackerOptions {
  onDragStart?: () => void;
  onDragMove?: (nextX: number, nextY: number, deltaDist: number) => void;
  onDragEnd?: (hasMoved: boolean, finalX: number, finalY: number) => void;
}

export function usePointerTracker({
  onDragStart,
  onDragMove,
  onDragEnd,
}: UsePointerTrackerOptions = {}) {
  const [isDragging, setIsDragging] = useState(false);

  const stateRef = useRef<PointerDragState>({
    isDown: false,
    hasMoved: false,
    startScreenX: 0,
    startScreenY: 0,
    initialWindowX: 0,
    initialWindowY: 0,
    bottomEdge: 1080,
    scale: 1,
  });

  // Tải trước số liệu màn hình và toạ độ ban đầu
  useEffect(() => {
    const updateMetrics = async () => {
      try {
        const appWin = getCurrentWindow();
        const [pos, monitor] = await Promise.all([
          appWin.outerPosition(),
          currentMonitor(),
        ]);
        stateRef.current.initialWindowX = pos.x;
        stateRef.current.initialWindowY = pos.y;
        if (monitor) {
          stateRef.current.scale = monitor.scaleFactor;
          stateRef.current.bottomEdge = monitor.position.y + monitor.size.height;
        }
      } catch {}
    };
    updateMetrics();
  }, []);

  const handlePointerDown = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      if (e.button !== 0) return;

      try {
        e.currentTarget.setPointerCapture(e.pointerId);
      } catch {}

      stateRef.current.isDown = true;
      stateRef.current.hasMoved = false;
      stateRef.current.startScreenX = e.screenX;
      stateRef.current.startScreenY = e.screenY;

      setIsDragging(true);
      if (onDragStart) onDragStart();

      try {
        const appWin = getCurrentWindow();
        const [pos, monitor] = await Promise.all([
          appWin.outerPosition(),
          currentMonitor(),
        ]);
        if (stateRef.current.isDown) {
          stateRef.current.initialWindowX = pos.x;
          stateRef.current.initialWindowY = pos.y;
          if (monitor) {
            stateRef.current.scale = monitor.scaleFactor;
            stateRef.current.bottomEdge = monitor.position.y + monitor.size.height;
          }
        }
      } catch {}
    },
    [onDragStart]
  );

  const handlePointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!stateRef.current.isDown) return;

      const dpr = window.devicePixelRatio || 1;
      const deltaX = Math.round((e.screenX - stateRef.current.startScreenX) * dpr);
      const deltaY = Math.round((e.screenY - stateRef.current.startScreenY) * dpr);

      const deltaDist = Math.hypot(deltaX, deltaY);
      if (deltaDist > 3) {
        stateRef.current.hasMoved = true;
      }

      const nextX = stateRef.current.initialWindowX + deltaX;
      const nextY = stateRef.current.initialWindowY + deltaY;

      if (onDragMove) {
        onDragMove(nextX, nextY, deltaDist);
      }
    },
    [onDragMove]
  );

  const handlePointerUp = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!stateRef.current.isDown) return;

      const hasMoved = stateRef.current.hasMoved;
      stateRef.current.isDown = false;
      setIsDragging(false);

      try {
        e.currentTarget.releasePointerCapture(e.pointerId);
      } catch {}

      if (onDragEnd) {
        onDragEnd(hasMoved, stateRef.current.initialWindowX, stateRef.current.initialWindowY);
      }
    },
    [onDragEnd]
  );

  return {
    isDragging,
    stateRef,
    pointerProps: {
      onPointerDown: handlePointerDown,
      onPointerMove: handlePointerMove,
      onPointerUp: handlePointerUp,
    },
  };
}
