import { useRef, useState, useCallback } from "react";
import {
  getCurrentWindow,
  currentMonitor,
  primaryMonitor,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

interface UseBubbleDragOptions {
  onBubbleClick?: () => void;
}

export const useBubbleDrag = ({ onBubbleClick }: UseBubbleDragOptions = {}) => {
  const [isDragging, setIsDragging] = useState(false);
  const dragRef = useRef<{
    isDown: boolean;
    hasMoved: boolean;
    startScreenX: number;
    startScreenY: number;
    initialWindowX: number;
    initialWindowY: number;
  }>({
    isDown: false,
    hasMoved: false,
    startScreenX: 0,
    startScreenY: 0,
    initialWindowX: 0,
    initialWindowY: 0,
  });

  const onPointerDown = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      // Chỉ nhận chuột trái
      if (e.button !== 0) return;

      const target = e.currentTarget;
      target.setPointerCapture(e.pointerId);

      let winX = 0;
      let winY = 0;

      try {
        const appWin = getCurrentWindow();
        const pos = await appWin.outerPosition();
        winX = pos.x;
        winY = pos.y;
      } catch {
        // Browser dev fallback
      }

      dragRef.current = {
        isDown: true,
        hasMoved: false,
        startScreenX: e.screenX,
        startScreenY: e.screenY,
        initialWindowX: winX,
        initialWindowY: winY,
      };

      setIsDragging(true);
    },
    []
  );

  const onPointerMove = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragRef.current.isDown) return;

      const deltaX = e.screenX - dragRef.current.startScreenX;
      const deltaY = e.screenY - dragRef.current.startScreenY;

      if (Math.hypot(deltaX, deltaY) > 4) {
        dragRef.current.hasMoved = true;
      }

      try {
        const appWin = getCurrentWindow();
        const nextX = dragRef.current.initialWindowX + deltaX;
        const nextY = dragRef.current.initialWindowY + deltaY;
        await appWin.setPosition(new PhysicalPosition(nextX, nextY));
      } catch {
        // Browser dev fallback
      }
    },
    []
  );

  const onPointerUp = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragRef.current.isDown) return;

      const hasMoved = dragRef.current.hasMoved;
      dragRef.current.isDown = false;
      setIsDragging(false);

      try {
        e.currentTarget.releasePointerCapture(e.pointerId);
      } catch {
        // Ignored
      }

      // 1. Nếu chỉ click nhẹ và không kéo di chuyển -> Tính là hành động Click
      if (!hasMoved) {
        onBubbleClick?.();
        return;
      }

      // 2. Nếu đã kéo thả -> Thực hiện Snap to Edge & Dismiss check
      try {
        const appWin = getCurrentWindow();
        const monitor =
          (await currentMonitor()) || (await primaryMonitor());

        if (monitor) {
          const currentPos = await appWin.outerPosition();
          const scale = monitor.scaleFactor || 1;
          const bubbleWidth = Math.round(64 * scale);
          const margin = Math.round(12 * scale);
          const taskbarMargin = Math.round(60 * scale);

          // Kiểm tra kéo sát đáy màn hình để ẩn (Dismiss to bottom)
          const bottomThreshold =
            monitor.position.y + monitor.size.height - Math.round(90 * scale);
          if (currentPos.y >= bottomThreshold) {
            await appWin.hide();
            return;
          }

          // Tính toạ độ hít viền trái hoặc phải
          const midX = monitor.position.x + monitor.size.width / 2;
          const targetX =
            currentPos.x + bubbleWidth / 2 < midX
              ? monitor.position.x + margin
              : monitor.position.x + monitor.size.width - bubbleWidth - margin;

          const minY = monitor.position.y + margin;
          const maxY =
            monitor.position.y + monitor.size.height - bubbleWidth - taskbarMargin;
          const targetY = Math.max(minY, Math.min(currentPos.y, maxY));

          // Hoạt ảnh trượt mượt mà (Smooth Snap Animation)
          const startX = currentPos.x;
          const startY = currentPos.y;
          const startTime = performance.now();
          const duration = 160; // ms

          const animateSnap = (now: number) => {
            const elapsed = now - startTime;
            const progress = Math.min(1, elapsed / duration);
            // Ease-out quad
            const ease = 1 - (1 - progress) * (1 - progress);
            const curX = Math.round(startX + (targetX - startX) * ease);
            const curY = Math.round(startY + (targetY - startY) * ease);

            appWin.setPosition(new PhysicalPosition(curX, curY));

            if (progress < 1) {
              requestAnimationFrame(animateSnap);
            } else {
              // Gọi lệnh Rust để bảo đảm toạ độ chuẩn tuyệt đối
              invoke("snap_bubble").catch(() => {});
            }
          };

          requestAnimationFrame(animateSnap);
        } else {
          // Gọi lệnh Rust dự phòng
          await invoke("snap_bubble");
        }
      } catch {
        console.log("Snap completed (fallback)");
      }
    },
    [onBubbleClick]
  );

  return {
    isDragging,
    dragProps: {
      onPointerDown,
      onPointerMove,
      onPointerUp,
    },
  };
};
