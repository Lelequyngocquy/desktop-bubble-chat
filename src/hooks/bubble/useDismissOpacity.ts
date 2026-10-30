/**
 * Pure calculation module for Bubble fade-out opacity when dragged near screen bottom (SRP).
 */
export interface DismissOpacityResult {
  opacity: number;
  isNearDismiss: boolean;
}

export function calculateDismissOpacity(
  nextY: number,
  bottomEdge: number,
  scale: number
): DismissOpacityResult {
  const dist = bottomEdge - nextY;

  // Vùng làm mờ: Bắt đầu mờ từ 340px đến ngưỡng dismiss 200px
  const startFadeDist = 340 * scale;
  const dismissThreshold = 200 * scale;
  const maxFadeDist = 140 * scale;

  let opacity = 1;
  const isNearDismiss = dist <= dismissThreshold;

  if (dist >= startFadeDist) {
    opacity = 1;
  } else if (dist <= maxFadeDist) {
    opacity = 0.25;
  } else {
    // Nội suy độ mờ tuyến tính từ 1.0 (ở startFadeDist) xuống 0.25 (ở maxFadeDist)
    const ratio = (startFadeDist - dist) / (startFadeDist - maxFadeDist);
    opacity = Math.round((1 - ratio * 0.75) * 100) / 100;
  }

  return { opacity, isNearDismiss };
}
