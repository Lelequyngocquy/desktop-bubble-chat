import { invoke } from "@tauri-apps/api/core";
import { IPC_COMMANDS, SnapResultPayload, BubblePositionPayload } from "../types/ipc";

export type SnapResult = SnapResultPayload;
export type BubblePosition = BubblePositionPayload;

/**
 * ScreenService acts as client gateway to the Rust backend positioning engine (SSOT).
 * All dimensions, collision avoidance, snap coordinates, and tuck ratios are managed exclusively by the backend.
 */
export class ScreenService {
  /**
   * Queries snap coordinates from Rust Backend with collision avoidance across multiple bubbles.
   */
  public static async getSnapCoords(
    currentX: number,
    currentY: number,
    accountIndex: number = 1
  ): Promise<SnapResult> {
    try {
      return await invoke<SnapResult>(IPC_COMMANDS.GET_BUBBLE_SNAP_COORDS, {
        currentX: Math.round(currentX),
        currentY: Math.round(currentY),
        accountIndex,
      });
    } catch {
      // Browser preview fallback
      const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
      const width = typeof window !== "undefined" ? window.screen.width * dpr : 1920;
      const isLeft = currentX + 32 * dpr < width / 2;
      return {
        targetX: isLeft ? -Math.round(64 * dpr * 0.3) : width - Math.round(64 * dpr * 0.7),
        targetY: Math.max(12 * dpr, currentY),
        shouldDismiss: false,
      };
    }
  }

  /**
   * Queries default initial coordinates from Rust Backend (stacking bubbles to prevent overlap).
   */
  public static async getDefaultCoords(accountIndex: number = 1): Promise<SnapResult> {
    try {
      return await invoke<SnapResult>(IPC_COMMANDS.GET_BUBBLE_DEFAULT_COORDS, { accountIndex });
    } catch {
      const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
      const height = typeof window !== "undefined" ? window.screen.height * dpr : 1080;
      return {
        targetX: -Math.round(56 * dpr * 0.75),
        targetY: Math.round(height / 3) + (accountIndex - 1) * Math.round(76 * dpr),
        shouldDismiss: false,
      };
    }
  }

  /**
   * Queries hover coordinates (appear_ratio 0.0 vs tuck_ratio).
   */
  public static async getHoverCoords(
    isHovered: boolean,
    accountIndex: number = 1
  ): Promise<SnapResult> {
    try {
      return await invoke<SnapResult>(IPC_COMMANDS.GET_BUBBLE_HOVER_COORDS, {
        isHovered,
        accountIndex,
      });
    } catch {
      const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
      const width = typeof window !== "undefined" ? window.screen.width * dpr : 1920;
      const bubbleW = Math.round(64 * dpr);
      const tuck = isHovered ? 0 : Math.round(bubbleW * 0.9);
      return {
        targetX: width - bubbleW + tuck,
        targetY: Math.round(1080 / 3) + (accountIndex - 1) * Math.round(76 * dpr),
        shouldDismiss: false,
      };
    }
  }

  /**
   * Queries current screen coordinates of a specific bubble.
   */
  public static async getBubblePosition(accountIndex: number = 1): Promise<{ x: number; y: number }> {
    return await invoke<{ x: number; y: number }>(IPC_COMMANDS.GET_BUBBLE_POSITION, { accountIndex });
  }

  /**
   * Queries coordinates of all currently visible bubbles.
   */
  public static async getAllBubblePositions(): Promise<BubblePosition[]> {
    return await invoke<BubblePosition[]>(IPC_COMMANDS.GET_ALL_BUBBLE_POSITIONS);
  }

  /**
   * Instructs backend to snap the bubble to screen edge.
   */
  public static async snapBubble(accountIndex: number = 1): Promise<void> {
    await invoke(IPC_COMMANDS.SNAP_BUBBLE, { accountIndex });
  }

  /**
   * Realtime check if a given coordinate overlaps with any other active bubble.
   */
  public static async checkCollision(
    accountIndex: number,
    candidateX: number,
    candidateY: number,
    bubbleSize: number = 64,
    gap: number = 12
  ): Promise<boolean> {
    try {
      const allBubbles = await this.getAllBubblePositions();
      return allBubbles.some((b) => {
        if (b.accountIndex === accountIndex || !b.isVisible) return false;
        const xOverlap = Math.abs(candidateX - b.x) < bubbleSize;
        const yOverlap = Math.abs(candidateY - b.y) < bubbleSize + gap;
        return xOverlap && yOverlap;
      });
    } catch {
      return false;
    }
  }
}
