import { useRef, useCallback, useEffect } from "react";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

/**
 * Handles smooth requestAnimationFrame spring/glide animations for the bubble window (SRP).
 */
export function useBubbleSpringAnimation() {
  const hoverAnimRef = useRef<number | null>(null);
  const snapAnimRef = useRef<number | null>(null);
  const moveAnimRef = useRef<number | null>(null);
  const latestMoveTargetRef = useRef<{ x: number; y: number } | null>(null);

  const cancelAllAnimations = useCallback(() => {
    if (hoverAnimRef.current) {
      cancelAnimationFrame(hoverAnimRef.current);
      hoverAnimRef.current = null;
    }
    if (snapAnimRef.current) {
      cancelAnimationFrame(snapAnimRef.current);
      snapAnimRef.current = null;
    }
    if (moveAnimRef.current) {
      cancelAnimationFrame(moveAnimRef.current);
      moveAnimRef.current = null;
    }
    latestMoveTargetRef.current = null;
  }, []);

  useEffect(() => {
    return cancelAllAnimations;
  }, [cancelAllAnimations]);

  // Animate hover slide (appear_ratio vs tuck_ratio)
  const animateHover = useCallback(
    (
      startX: number,
      startY: number,
      targetX: number,
      targetY: number,
      duration: number = 180,
      onComplete?: () => void
    ) => {
      if (hoverAnimRef.current) {
        cancelAnimationFrame(hoverAnimRef.current);
      }

      const startTime = performance.now();
      const appWin = getCurrentWindow();

      const step = (now: number) => {
        const elapsed = now - startTime;
        const progress = Math.min(1, elapsed / duration);
        // Ease-out cubic: 1 - (1 - progress)^3
        const ease = 1 - Math.pow(1 - progress, 3);
        const curX = Math.round(startX + (targetX - startX) * ease);
        const curY = Math.round(startY + (targetY - startY) * ease);

        appWin.setPosition(new PhysicalPosition(curX, curY));

        if (progress < 1) {
          hoverAnimRef.current = requestAnimationFrame(step);
        } else {
          hoverAnimRef.current = null;
          if (onComplete) onComplete();
        }
      };

      hoverAnimRef.current = requestAnimationFrame(step);
    },
    []
  );

  // Animate snap after drag release
  const animateSnap = useCallback(
    (
      startX: number,
      startY: number,
      targetX: number,
      targetY: number,
      startOpacity: number,
      duration: number = 160,
      onFrame?: (opacity: number) => void,
      onComplete?: () => void
    ) => {
      if (snapAnimRef.current) {
        cancelAnimationFrame(snapAnimRef.current);
      }

      const startTime = performance.now();
      const appWin = getCurrentWindow();

      const step = (now: number) => {
        const elapsed = now - startTime;
        const progress = Math.min(1, elapsed / duration);
        // Ease-out quad
        const ease = 1 - (1 - progress) * (1 - progress);
        const curX = Math.round(startX + (targetX - startX) * ease);
        const curY = Math.round(startY + (targetY - startY) * ease);
        const curOpacity = Math.round((startOpacity + (1 - startOpacity) * ease) * 100) / 100;

        if (onFrame) onFrame(curOpacity);
        appWin.setPosition(new PhysicalPosition(curX, curY));

        if (progress < 1) {
          snapAnimRef.current = requestAnimationFrame(step);
        } else {
          snapAnimRef.current = null;
          if (onComplete) onComplete();
        }
      };

      snapAnimRef.current = requestAnimationFrame(step);
    },
    []
  );

  // Throttled position update during drag to prevent IPC bottleneck without dropping latest target
  const scheduleMove = useCallback((x: number, y: number) => {
    latestMoveTargetRef.current = { x, y };
    if (!moveAnimRef.current) {
      moveAnimRef.current = requestAnimationFrame(async () => {
        moveAnimRef.current = null;
        const target = latestMoveTargetRef.current;
        if (!target) return;
        try {
          const appWin = getCurrentWindow();
          await appWin.setPosition(new PhysicalPosition(target.x, target.y));
        } catch {
          // Dev preview fallback
        }
      });
    }
  }, []);

  return {
    animateHover,
    animateSnap,
    scheduleMove,
    cancelAllAnimations,
  };
}
