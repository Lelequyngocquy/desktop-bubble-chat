use tauri::PhysicalPosition;

use super::collision::{BubbleRect, CollisionResolver, ColumnBubbleSlot};
use super::config::{BubblePhysicsConfig, ScreenMetrics, SnapResult};

/// Result of a mutual snap operation including all displaced column slots
#[derive(Debug, Clone)]
pub struct MutualSnapResult {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
    pub is_left_edge: bool,
    pub column_slots: Vec<ColumnBubbleSlot>,
}

pub struct BubbleMovementEngine;

impl BubbleMovementEngine {
    /// Computes X coordinate along left or right edge based on tuck ratio
    pub fn calculate_edge_x(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        current_x: i32,
        ratio: f64,
    ) -> i32 {
        let scale = screen.scale_factor;
        let bubble_width = (config.size * scale) as i32;
        let tuck = (bubble_width as f64 * ratio) as i32;
        let mid_x = screen.x + screen.width / 2;
        if current_x + bubble_width / 2 < mid_x {
            screen.x - tuck
        } else {
            screen.x + screen.width - bubble_width + tuck
        }
    }

    /// Computes edge snap with dismiss check and full mutual column layout resolution
    pub fn calculate_mutual_snap(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        win_pos: PhysicalPosition<i32>,
        moving_index: u32,
        all_bubbles: &[BubbleRect],
    ) -> MutualSnapResult {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let dismiss_margin = (config.dismiss_margin * scale) as i32;
        let mid_x = screen.x + screen.width / 2;
        let is_left_edge = win_pos.x + bubble_size / 2 < mid_x;
        let dismiss_threshold = screen.y + screen.height - dismiss_margin;
        if win_pos.y + bubble_size >= dismiss_threshold {
            return MutualSnapResult {
                target_x: win_pos.x,
                target_y: win_pos.y,
                should_dismiss: true,
                is_left_edge,
                column_slots: Vec::new(),
            };
        }
        let target_x = Self::calculate_edge_x(screen, config, win_pos.x, config.tuck_ratio);
        let column_slots = CollisionResolver::resolve_mutual_column_layout(
            moving_index,
            win_pos.y,
            is_left_edge,
            screen,
            config,
            all_bubbles,
        );
        let target_y = column_slots
            .iter()
            .find(|slot| slot.account_index == moving_index)
            .map(|slot| slot.target_y)
            .unwrap_or(win_pos.y);
        MutualSnapResult {
            target_x,
            target_y,
            should_dismiss: false,
            is_left_edge,
            column_slots,
        }
    }

    /// Computes edge snap with dismiss threshold check and collision resolution
    pub fn calculate_snap(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        win_pos: PhysicalPosition<i32>,
        moving_index: u32,
        all_bubbles: &[BubbleRect],
    ) -> SnapResult {
        let res = Self::calculate_mutual_snap(screen, config, win_pos, moving_index, all_bubbles);
        SnapResult {
            target_x: res.target_x,
            target_y: res.target_y,
            should_dismiss: res.should_dismiss,
            is_left_edge: res.is_left_edge,
        }
    }

    /// Computes hover coordinates when entering or exiting bubble
    pub fn calculate_hover_pos(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        current_x: i32,
        current_y: i32,
        is_hovered: bool,
    ) -> PhysicalPosition<i32> {
        let ratio = if is_hovered {
            config.appear_ratio
        } else {
            config.tuck_ratio
        };
        let target_x = Self::calculate_edge_x(screen, config, current_x, ratio);
        let scale = screen.scale_factor;
        let bubble_width = (config.size * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_width - taskbar_margin).max(min_y);
        let target_y = current_y.clamp(min_y, max_y);
        PhysicalPosition {
            x: target_x,
            y: target_y,
        }
    }

    /// Computes stacked position for newly spawned bubble to avoid overlapping
    pub fn calculate_stacked_pos(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        existing_bubbles: &[BubbleRect],
    ) -> PhysicalPosition<i32> {
        let mid_x = screen.x + screen.width / 2;
        let is_left_edge = if let Some(first_b) = existing_bubbles.first() {
            first_b.x + first_b.width / 2 < mid_x
        } else {
            false
        };
        let initial_x = if is_left_edge {
            screen.x
        } else {
            screen.x + screen.width
        };
        let target_x = Self::calculate_edge_x(
            screen,
            config,
            initial_x,
            config.appear_ratio,
        );
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let gap = (config.gap * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
        let default_y = (screen.y + screen.height / 3).clamp(min_y, max_y);
        let same_edge_bubbles: Vec<&BubbleRect> = existing_bubbles
            .iter()
            .filter(|b| (b.x + b.width / 2 < mid_x) == is_left_edge)
            .collect();
        let desired_y = if let Some(last_b) = same_edge_bubbles.iter().max_by_key(|b| b.y) {
            last_b.y + last_b.height + gap
        } else {
            default_y
        };
        let layout = CollisionResolver::resolve_mutual_column_layout(
            u32::MAX,
            desired_y,
            is_left_edge,
            screen,
            config,
            existing_bubbles,
        );
        let target_y = layout
            .iter()
            .find(|slot| slot.account_index == u32::MAX)
            .map(|slot| slot.target_y)
            .unwrap_or_else(|| desired_y.clamp(min_y, max_y));
        PhysicalPosition {
            x: target_x,
            y: target_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mutual_snap_bottom_dismiss() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        // Screen bottom is 1080. dismiss_margin is 160. Threshold is 1080 - 160 = 920.
        // Dropping at Y = 950 with size 56 gives 950 + 56 = 1006 >= 920.
        let pos = PhysicalPosition { x: 1800, y: 950 };
        let snap = BubbleMovementEngine::calculate_mutual_snap(
            &screen,
            &config,
            pos,
            1,
            &[],
        );
        assert!(snap.should_dismiss);
    }

    #[test]
    fn test_calculate_mutual_snap_normal_glide() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let pos = PhysicalPosition { x: 1800, y: 500 };
        let snap = BubbleMovementEngine::calculate_mutual_snap(
            &screen,
            &config,
            pos,
            1,
            &[],
        );
        assert!(!snap.should_dismiss);
    }
}
