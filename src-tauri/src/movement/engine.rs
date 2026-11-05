use tauri::PhysicalPosition;

use super::collision::{BubbleRect, CollisionResolver};
use super::config::{BubblePhysicsConfig, ScreenMetrics, SnapResult};

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

    /// Computes edge snap with dismiss threshold check and collision resolution
    pub fn calculate_snap(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        win_pos: PhysicalPosition<i32>,
        moving_index: u32,
        all_bubbles: &[BubbleRect],
    ) -> SnapResult {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let dismiss_margin = (config.dismiss_margin * scale) as i32;
        let mid_x = screen.x + screen.width / 2;
        let is_left_edge = win_pos.x + bubble_size / 2 < mid_x;
        let dismiss_threshold = screen.y + screen.height - dismiss_margin;
        if win_pos.y + bubble_size >= dismiss_threshold {
            return SnapResult {
                target_x: win_pos.x,
                target_y: win_pos.y,
                should_dismiss: true,
                is_left_edge,
            };
        }
        let target_x = Self::calculate_edge_x(screen, config, win_pos.x, config.tuck_ratio);
        let resolved_y = CollisionResolver::resolve_multi_bubble_collision(
            moving_index,
            win_pos.y,
            is_left_edge,
            screen,
            config,
            all_bubbles,
        );
        SnapResult {
            target_x,
            target_y: resolved_y,
            should_dismiss: false,
            is_left_edge,
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
        let is_left_edge = true;
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
        let target_y = CollisionResolver::find_free_stacked_slot(
            screen,
            config,
            existing_bubbles,
            is_left_edge,
        );
        PhysicalPosition {
            x: target_x,
            y: target_y,
        }
    }
}
