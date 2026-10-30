import { getAccountThemeColor, AccountThemeColor } from "../constants/accountColors";
import { ScreenService, SnapResult } from "../services/screenService";

export type BubbleMotionState = "IDLE" | "HOVERED" | "DRAGGING" | "SNAPPING" | "DISMISSED";
export type ScreenEdge = "left" | "right";

export interface BubblePoint {
  x: number;
  y: number;
}

/**
 * BubbleEntity đóng gói trạng thái và hành vi của một Bong bóng chat độc lập (OOP Pattern).
 * Mỗi cửa sổ bubble (bubble-01 .. bubble-05) được mô hình hóa bằng 1 instance của entity này.
 */
export class BubbleEntity {
  public readonly accountIndex: number;
  public readonly windowLabel: string;
  public readonly themeColor: AccountThemeColor;

  private _position: BubblePoint;
  private _targetPosition: BubblePoint;
  private _edge: ScreenEdge;
  private _motionState: BubbleMotionState;
  private _unreadCount: number;

  constructor(accountIndex: number = 1, initialPosition?: BubblePoint) {
    this.accountIndex = accountIndex;
    this.windowLabel = `bubble-${accountIndex.toString().padStart(2, "0")}`;
    this.themeColor = getAccountThemeColor(accountIndex);

    this._position = initialPosition || { x: 0, y: 0 };
    this._targetPosition = { ...this._position };
    this._edge = "right";
    this._motionState = "IDLE";
    this._unreadCount = 0;
  }

  // Getters
  public get position(): BubblePoint {
    return { ...this._position };
  }

  public get targetPosition(): BubblePoint {
    return { ...this._targetPosition };
  }

  public get edge(): ScreenEdge {
    return this._edge;
  }

  public get motionState(): BubbleMotionState {
    return this._motionState;
  }

  public get unreadCount(): number {
    return this._unreadCount;
  }

  public get isDragging(): boolean {
    return this._motionState === "DRAGGING";
  }

  // Setters
  public setPosition(pos: BubblePoint): void {
    this._position = { ...pos };
  }

  public setUnreadCount(count: number): void {
    this._unreadCount = Math.max(0, count);
  }

  // Behaviors (Hành vi OOP)
  public startDrag(): void {
    this._motionState = "DRAGGING";
  }

  public updateDragPosition(pos: BubblePoint): void {
    this._position = { ...pos };
    this._motionState = "DRAGGING";
  }

  public async snap(): Promise<SnapResult> {
    this._motionState = "SNAPPING";
    const snapResult = await ScreenService.getSnapCoords(
      this._position.x,
      this._position.y,
      this.accountIndex
    );

    this._targetPosition = { x: snapResult.targetX, y: snapResult.targetY };
    if (snapResult.isLeftEdge !== undefined) {
      this._edge = snapResult.isLeftEdge ? "left" : "right";
    } else {
      const midX = typeof window !== "undefined" ? window.screen.width / 2 : 960;
      this._edge = snapResult.targetX < midX ? "left" : "right";
    }

    if (snapResult.shouldDismiss) {
      this._motionState = "DISMISSED";
    } else {
      this._motionState = "IDLE";
    }

    return snapResult;
  }

  public async getHoverCoords(isHovered: boolean): Promise<SnapResult> {
    this._motionState = isHovered ? "HOVERED" : "IDLE";
    const snapResult = await ScreenService.getHoverCoords(isHovered, this.accountIndex);
    if (snapResult.isLeftEdge !== undefined) {
      this._edge = snapResult.isLeftEdge ? "left" : "right";
    }
    return snapResult;
  }

  public dismiss(): void {
    this._motionState = "DISMISSED";
  }
}
