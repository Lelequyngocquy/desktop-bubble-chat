import { useRef, useState, useCallback, useEffect } from "react";
import { currentMonitor, getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ScreenService } from "../services";

interface UseBubbleDragOptions {
  onBubbleClick?: () => void;
  isChatOpen?: boolean;
}

export const useBubbleDrag = ({ onBubbleClick, isChatOpen = false }: UseBubbleDragOptions = {}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [dragOpacity, setDragOpacity] = useState(1);
  const [isNearDismiss, setIsNearDismiss] = useState(false);

  const hoverAnimRef = useRef<number | null>(null);
  const isHoveredRef = useRef<boolean>(false);

  const dragRef = useRef<{
    isDown: boolean;
    hasMoved: boolean;
    startScreenX: number;
    startScreenY: number;
    initialWindowX: number;
    initialWindowY: number;
    bottomEdge: number;
    scale: number;
    lastOpacity: number;
    lastNearDismiss: boolean;
  }>({
    isDown: false,
    hasMoved: false,
    startScreenX: 0,
    startScreenY: 0,
    initialWindowX: 0,
    initialWindowY: 0,
    bottomEdge: 1080,
    scale: 1,
    lastOpacity: 1,
    lastNearDismiss: false,
  });

  // Điều khiển hoạt ảnh trượt ra (appear_ratio 0.0) / trượt vào (tuck_ratio)
  const setHoverState = useCallback(
    async (hovered: boolean) => {
      // Tuyệt đối không can thiệp nếu đang giữ kéo chuột
      if (dragRef.current.isDown) return;

      // Nếu cửa sổ chat đang mở, bong bóng luôn giữ trạng thái kéo ra toàn bộ (0.0)
      const targetHovered = isChatOpen ? true : hovered;

      if (isHoveredRef.current === targetHovered) return;
      isHoveredRef.current = targetHovered;

      try {
        const appWin = getCurrentWindow();
        const currentPos = await appWin.outerPosition();
        const { targetX, targetY } = await ScreenService.getHoverCoords(targetHovered);

        if (hoverAnimRef.current) {
          cancelAnimationFrame(hoverAnimRef.current);
          hoverAnimRef.current = null;
        }

        const startX = currentPos.x;
        const startY = currentPos.y;
        const startTime = performance.now();
        const duration = 140; // ms

        const animateHover = (now: number) => {
          if (dragRef.current.isDown) return;

          const elapsed = now - startTime;
          const progress = Math.min(1, elapsed / duration);
          // Ease-out quad
          const ease = 1 - (1 - progress) * (1 - progress);
          const curX = Math.round(startX + (targetX - startX) * ease);
          const curY = Math.round(startY + (targetY - startY) * ease);

          appWin.setPosition(new PhysicalPosition(curX, curY));

          if (progress < 1) {
            hoverAnimRef.current = requestAnimationFrame(animateHover);
          } else {
            hoverAnimRef.current = null;
          }
        };

        hoverAnimRef.current = requestAnimationFrame(animateHover);
      } catch {
        // Fallback
      }
    },
    [isChatOpen]
  );

  // Lắng nghe sự kiện chuột ra/vào trên toàn bộ WebView2 window
  useEffect(() => {
    const handleMouseLeave = () => {
      setHoverState(false);
    };
    const handleMouseEnter = () => {
      setHoverState(true);
    };
    window.addEventListener("mouseleave", handleMouseLeave);
    window.addEventListener("mouseenter", handleMouseEnter);
    return () => {
      window.removeEventListener("mouseleave", handleMouseLeave);
      window.removeEventListener("mouseenter", handleMouseEnter);
    };
  }, [setHoverState]);

  // Đồng bộ vị trí bong bóng khi mở hoặc đóng khung chat
  useEffect(() => {
    if (isChatOpen) {
      setHoverState(true);
    } else {
      isHoveredRef.current = false;
      setHoverState(false);
    }
  }, [isChatOpen, setHoverState]);

  const onPointerDown = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      // Chỉ nhận chuột trái
      if (e.button !== 0) return;

      // Hủy hoạt ảnh hover nếu bắt đầu kéo
      if (hoverAnimRef.current) {
        cancelAnimationFrame(hoverAnimRef.current);
        hoverAnimRef.current = null;
      }

      const target = e.currentTarget;
      target.setPointerCapture(e.pointerId);

      let winX = 0;
      let winY = 0;
      let bottomEdge = (window.screen.height || 1080) * (window.devicePixelRatio || 1);
      let scale = window.devicePixelRatio || 1;

      try {
        const appWin = getCurrentWindow();
        const [pos, monitor] = await Promise.all([
          appWin.outerPosition(),
          currentMonitor(),
        ]);
        winX = pos.x;
        winY = pos.y;
        if (monitor) {
          scale = monitor.scaleFactor;
          bottomEdge = monitor.position.y + monitor.size.height;
        }
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
        bottomEdge,
        scale,
        lastOpacity: 1,
        lastNearDismiss: false,
      };

      setIsDragging(true);
      setDragOpacity(1);
      setIsNearDismiss(false);
    },
    []
  );

  const onPointerMove = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragRef.current.isDown) return;

      const dpr = window.devicePixelRatio || 1;
      const deltaX = Math.round((e.screenX - dragRef.current.startScreenX) * dpr);
      const deltaY = Math.round((e.screenY - dragRef.current.startScreenY) * dpr);

      if (Math.hypot(deltaX, deltaY) > 4) {
        dragRef.current.hasMoved = true;
      }

      const nextX = dragRef.current.initialWindowX + deltaX;
      const nextY = dragRef.current.initialWindowY + deltaY;

      // Tính khoảng cách từ đỉnh cửa sổ tới đáy màn hình
      const scale = dragRef.current.scale;
      const dist = dragRef.current.bottomEdge - nextY;

      // Vùng làm mờ: Bắt đầu mờ từ 340px đến ngưỡng dismiss 200px
      const startFadeDist = 340 * scale;
      const dismissThreshold = 200 * scale;
      const maxFadeDist = 140 * scale;

      let newOpacity = 1;
      const nearDismiss = dist <= dismissThreshold;

      if (dist >= startFadeDist) {
        newOpacity = 1;
      } else if (dist <= maxFadeDist) {
        newOpacity = 0.25;
      } else {
        // Nội suy độ mờ tuyến tính từ 1.0 (ở startFadeDist) xuống 0.25 (ở maxFadeDist)
        const ratio = (startFadeDist - dist) / (startFadeDist - maxFadeDist);
        newOpacity = Math.round((1 - ratio * 0.75) * 100) / 100;
      }

      if (newOpacity !== dragRef.current.lastOpacity) {
        dragRef.current.lastOpacity = newOpacity;
        setDragOpacity(newOpacity);
      }

      if (nearDismiss !== dragRef.current.lastNearDismiss) {
        dragRef.current.lastNearDismiss = nearDismiss;
        setIsNearDismiss(nearDismiss);
      }

      try {
        const appWin = getCurrentWindow();
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
        setDragOpacity(1);
        setIsNearDismiss(false);
        dragRef.current.lastOpacity = 1;
        dragRef.current.lastNearDismiss = false;
        onBubbleClick?.();
        return;
      }

      // 2. Nếu đã kéo thả -> Hỏi Rust Backend toạ độ Snap (Single Source of Truth)
      try {
        const appWin = getCurrentWindow();
        const currentPos = await appWin.outerPosition();
        const { targetX, targetY, shouldDismiss } = await ScreenService.getSnapCoords(
          currentPos.x,
          currentPos.y
        );

        if (shouldDismiss) {
          await appWin.hide();
          await invoke("toggle_bubble", { visible: false });
          setDragOpacity(1);
          setIsNearDismiss(false);
          dragRef.current.lastOpacity = 1;
          dragRef.current.lastNearDismiss = false;
          return;
        }

        // Hoạt ảnh trượt mượt mà (Smooth Snap Animation) & hồi phục độ mờ về 1.0
        const startX = currentPos.x;
        const startY = currentPos.y;
        const startOpacity = dragRef.current.lastOpacity;
        const startTime = performance.now();
        const duration = 160; // ms

        const animateSnap = (now: number) => {
          const elapsed = now - startTime;
          const progress = Math.min(1, elapsed / duration);
          // Ease-out quad
          const ease = 1 - (1 - progress) * (1 - progress);
          const curX = Math.round(startX + (targetX - startX) * ease);
          const curY = Math.round(startY + (targetY - startY) * ease);
          const curOpacity = Math.round((startOpacity + (1 - startOpacity) * ease) * 100) / 100;

          setDragOpacity(curOpacity);
          appWin.setPosition(new PhysicalPosition(curX, curY));

          if (progress < 1) {
            requestAnimationFrame(animateSnap);
          } else {
            setDragOpacity(1);
            setIsNearDismiss(false);
            dragRef.current.lastOpacity = 1;
            dragRef.current.lastNearDismiss = false;
            isHoveredRef.current = false;
            // Gọi lệnh Rust để bảo đảm toạ độ chuẩn tuyệt đối
            invoke("snap_bubble").catch(() => {});
          }
        };

        requestAnimationFrame(animateSnap);
      } catch {
        setDragOpacity(1);
        setIsNearDismiss(false);
        dragRef.current.lastOpacity = 1;
        dragRef.current.lastNearDismiss = false;
        isHoveredRef.current = false;
        // Gọi lệnh Rust dự phòng
        await invoke("snap_bubble").catch(() => {});
      }
    },
    [onBubbleClick]
  );

  return {
    isDragging,
    dragOpacity,
    isNearDismiss,
    dragProps: {
      onPointerDown,
      onPointerMove,
      onPointerUp,
      onMouseEnter: () => setHoverState(true),
      onMouseLeave: () => setHoverState(false),
    },
  };
};
