import { useRef, useState, useCallback, useEffect, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { BubbleEntity } from "../../models/BubbleEntity";
import { calculateDismissOpacity } from "./useDismissOpacity";
import { useBubbleSpringAnimation } from "./useBubbleSpringAnimation";
import { usePointerTracker } from "./usePointerTracker";

interface UseBubbleDragOptions {
  isChatOpen?: boolean;
  accountIndex?: number;
}

/**
 * Facade Hook coordinating Bubble motion, pointer interactions, and spring animations.
 * Refactored to follow Single Responsibility Principle (SRP) and OOP Entity patterns.
 */
export const useBubbleDrag = ({
  isChatOpen = false,
  accountIndex = 1,
}: UseBubbleDragOptions = {}) => {
  const bubbleEntity = useMemo(() => new BubbleEntity(accountIndex), [accountIndex]);

  const [dragOpacity, setDragOpacity] = useState(1);
  const [isNearDismiss, setIsNearDismiss] = useState(false);
  const isHoveredRef = useRef<boolean>(false);
  const currentPosRef = useRef<{ x: number; y: number }>({ x: 0, y: 0 });

  const { animateHover, animateSnap, scheduleMove, cancelAllAnimations } =
    useBubbleSpringAnimation();

  // Điều khiển hoạt ảnh trượt vào / trượt ra mép màn hình
  const setHoverState = useCallback(
    async (hovered: boolean, force: boolean = false) => {
      const targetHovered = isChatOpen ? true : hovered;
      if (!force && isHoveredRef.current === targetHovered) return;
      isHoveredRef.current = targetHovered;

      try {
        const appWin = getCurrentWindow();
        const currentPos = await appWin.outerPosition();
        const { targetX, targetY } = await bubbleEntity.getHoverCoords(targetHovered);

        animateHover(currentPos.x, currentPos.y, targetX, targetY);
      } catch {
        // Fallback
      }
    },
    [isChatOpen, bubbleEntity, animateHover]
  );

  // Lắng nghe chuột ra vào cửa sổ
  useEffect(() => {
    const handleMouseLeave = () => setHoverState(false);
    const handleMouseEnter = () => setHoverState(true);

    window.addEventListener("mouseleave", handleMouseLeave);
    window.addEventListener("mouseenter", handleMouseEnter);
    return () => {
      window.removeEventListener("mouseleave", handleMouseLeave);
      window.removeEventListener("mouseenter", handleMouseEnter);
    };
  }, [setHoverState]);

  // Đồng bộ vị trí bong bóng khi mở hoặc đóng khung chat
  useEffect(() => {
    setHoverState(isChatOpen, true);
  }, [isChatOpen, setHoverState]);

  // Trình theo dõi con trỏ chuột kéo thả (Pointer Tracker)
  const { isDragging, stateRef, pointerProps } = usePointerTracker({
    onDragStart: () => {
      cancelAllAnimations();
      bubbleEntity.startDrag();
      setDragOpacity(1);
      setIsNearDismiss(false);
    },
    onDragMove: (nextX, nextY) => {
      currentPosRef.current = { x: nextX, y: nextY };
      bubbleEntity.updateDragPosition({ x: nextX, y: nextY });

      const { opacity, isNearDismiss: nearDismiss } = calculateDismissOpacity(
        nextY,
        stateRef.current.bottomEdge,
        stateRef.current.scale
      );

      setDragOpacity(opacity);
      setIsNearDismiss(nearDismiss);
      scheduleMove(nextX, nextY);
    },
    onDragEnd: async (hasMoved) => {
      if (!hasMoved) {
        setDragOpacity(1);
        setIsNearDismiss(false);
        await invoke("toggle_chat", { accountIndex }).catch(() => {});
        return;
      }

      try {
        const appWin = getCurrentWindow();
        const currentPos = await appWin.outerPosition();
        bubbleEntity.setPosition({ x: currentPos.x, y: currentPos.y });

        // Hỏi Rust Backend toạ độ Snap với cơ chế chống va chạm (SSOT)
        const snapResult = await bubbleEntity.snap();

        // Nếu kéo sát đáy màn hình -> Ẩn về Tray
        if (snapResult.shouldDismiss) {
          await appWin.hide();
          setDragOpacity(1);
          setIsNearDismiss(false);
          await invoke("toggle_bubble", { visible: false, accountIndex }).catch(() => {});
          return;
        }

        // Hoạt ảnh trượt mượt mà về mép màn hình
        animateSnap(
          currentPos.x,
          currentPos.y,
          snapResult.targetX,
          snapResult.targetY,
          dragOpacity,
          160,
          (opacity) => setDragOpacity(opacity),
          () => {
            setDragOpacity(1);
            setIsNearDismiss(false);
            isHoveredRef.current = false;
            invoke("snap_bubble", { accountIndex }).catch(() => {});
          }
        );
      } catch {
        setDragOpacity(1);
        setIsNearDismiss(false);
        isHoveredRef.current = false;
        await invoke("snap_bubble", { accountIndex }).catch(() => {});
      }
    },
  });

  return {
    isDragging,
    dragOpacity,
    isNearDismiss,
    dragProps: {
      ...pointerProps,
      onMouseEnter: () => setHoverState(true),
      onMouseLeave: () => setHoverState(false),
    },
  };
};
