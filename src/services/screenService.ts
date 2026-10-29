import { invoke } from "@tauri-apps/api/core";

export interface SnapResult {
  targetX: number;
  targetY: number;
  shouldDismiss: boolean;
}

/**
 * ScreenService đóng vai trò là Client kết nối với Rust Backend (Single Source of Truth).
 * Toàn bộ hằng số kích thước, tỉ lệ lấn mép và công thức toán học do Backend quản lý độc quyền.
 */
export class ScreenService {
  /**
   * Truy vấn tọa độ hít mép từ Rust Backend
   */
  public static async getSnapCoords(currentX: number, currentY: number): Promise<SnapResult> {
    try {
      const res = await invoke<SnapResult>("get_bubble_snap_coords", {
        currentX: Math.round(currentX),
        currentY: Math.round(currentY),
      });
      return res;
    } catch {
      // Fallback môi trường browser preview dev mode
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
   * Truy vấn tọa độ xuất hiện mặc định từ Rust Backend (mép phải, 1/3 chiều cao màn hình)
   */
  public static async getDefaultCoords(): Promise<SnapResult> {
    try {
      const res = await invoke<SnapResult>("get_bubble_default_coords");
      return res;
    } catch {
      const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
      const width = typeof window !== "undefined" ? window.screen.width * dpr : 1920;
      const height = typeof window !== "undefined" ? window.screen.height * dpr : 1080;
      return {
        targetX: width - Math.round(64 * dpr * 0.1),
        targetY: Math.round(height / 3),
        shouldDismiss: false,
      };
    }
  }

  /**
   * Truy vấn tọa độ khi hover vào/ra khỏi bong bóng từ Rust Backend (appear_ratio 0.0 vs tuck_ratio)
   */
  public static async getHoverCoords(isHovered: boolean): Promise<SnapResult> {
    try {
      const res = await invoke<SnapResult>("get_bubble_hover_coords", {
        isHovered,
      });
      return res;
    } catch {
      const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
      const width = typeof window !== "undefined" ? window.screen.width * dpr : 1920;
      const bubbleW = Math.round(64 * dpr);
      const tuck = isHovered ? 0 : Math.round(bubbleW * 0.9);
      return {
        targetX: width - bubbleW + tuck,
        targetY: Math.round(1080 / 3),
        shouldDismiss: false,
      };
    }
  }
}
