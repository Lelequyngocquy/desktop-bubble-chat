use tauri::PhysicalPosition;

use super::collision::{BubbleRect, CollisionResolver, ColumnBubbleSlot};
use super::config::{BubblePhysicsConfig, ScreenMetrics, SnapResult};

/// Interaction and alert flags governing bubble edge-tuck eligibility
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BubbleInteractionFlags {
    pub is_hovered: bool,
    pub is_dragging: bool,
    pub is_chat_open: bool,
    pub is_chat_pinned: bool,
    pub is_free_position: bool,
    pub is_alert_active: bool,
}

/// Reason why a bubble is untucked from the screen edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UntuckMode {
    UserInteraction,
    AlertReveal,
}

/// Unified drag-drop resolution plan shared across Windows, macOS, and Linux
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BubbleDropPlan {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
    pub displaced_siblings: Vec<(u32, i32, i32)>,
}

/// Computed edge tuck/untuck transition coordinates for target and displaced siblings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverTransitionPlan {
    pub target_x: i32,
    pub target_y: i32,
    pub displaced_siblings: Vec<(u32, i32, i32)>,
}

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
    /// Returns true if interaction or alert state requires bubble to remain untucked
    pub fn should_stay_untucked(flags: BubbleInteractionFlags) -> bool {
        flags.is_hovered || flags.is_chat_open || flags.is_chat_pinned || flags.is_alert_active
    }

    /// Returns true when bubble is eligible to physically tuck into screen edge
    pub fn can_auto_tuck(flags: BubbleInteractionFlags) -> bool {
        !flags.is_free_position && !flags.is_dragging && !Self::should_stay_untucked(flags)
    }

    /// Resolves edge tuck ratio based on whether bubble should remain untucked
    pub fn effective_edge_ratio(config: &BubblePhysicsConfig, stay_untucked: bool) -> f64 {
        if stay_untucked { config.appear_ratio } else { config.tuck_ratio }
    }

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

    /// Computes edge snap with custom edge ratio, dismiss check, and mutual column layout
    pub fn calculate_mutual_snap_with_ratio(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        win_pos: PhysicalPosition<i32>,
        moving_index: u32,
        all_bubbles: &[BubbleRect],
        edge_ratio: f64,
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
        let target_x = Self::calculate_edge_x(screen, config, win_pos.x, edge_ratio);
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

    /// Computes edge snap with dismiss check and full mutual column layout resolution
    pub fn calculate_mutual_snap(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        win_pos: PhysicalPosition<i32>,
        moving_index: u32,
        all_bubbles: &[BubbleRect],
    ) -> MutualSnapResult {
        Self::calculate_mutual_snap_with_ratio(screen, config, win_pos, moving_index, all_bubbles, config.tuck_ratio)
    }

    /// Computes edge snap with custom edge ratio and collision resolution
    pub fn calculate_snap_with_ratio(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        win_pos: PhysicalPosition<i32>,
        moving_index: u32,
        all_bubbles: &[BubbleRect],
        edge_ratio: f64,
    ) -> SnapResult {
        let res = Self::calculate_mutual_snap_with_ratio(screen, config, win_pos, moving_index, all_bubbles, edge_ratio);
        SnapResult {
            target_x: res.target_x,
            target_y: res.target_y,
            should_dismiss: res.should_dismiss,
            is_left_edge: res.is_left_edge,
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
        Self::calculate_snap_with_ratio(screen, config, win_pos, moving_index, all_bubbles, config.tuck_ratio)
    }

    /// Computes unified drag-drop plan preserving each displaced sibling's own X coordinate
    pub fn compute_drop_plan(
        account_index: u32,
        x: i32,
        y: i32,
        stay_untucked: bool,
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        all_bubbles: &[BubbleRect],
    ) -> BubbleDropPlan {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        if config.is_free_position {
            let dismiss_margin = (config.dismiss_margin * scale) as i32;
            let dismiss_threshold = screen.y + screen.height - dismiss_margin;
            if y + bubble_size >= dismiss_threshold {
                return BubbleDropPlan { target_x: x, target_y: y, should_dismiss: true, displaced_siblings: Vec::new() };
            }
            let (target_x, target_y) = CollisionResolver::resolve_2d_free_collision(account_index, x, y, screen, config, all_bubbles);
            return BubbleDropPlan { target_x, target_y, should_dismiss: false, displaced_siblings: Vec::new() };
        }
        let edge_ratio = Self::effective_edge_ratio(config, stay_untucked);
        let snap = Self::calculate_mutual_snap_with_ratio(screen, config, PhysicalPosition { x, y }, account_index, all_bubbles, edge_ratio);
        let displaced_siblings = if snap.should_dismiss {
            Vec::new()
        } else {
            snap.column_slots
                .into_iter()
                .filter(|s| s.account_index != account_index)
                .map(|s| {
                    let sib_x = all_bubbles.iter().find(|b| b.account_index == s.account_index).map(|b| b.x).unwrap_or(snap.target_x);
                    (s.account_index, sib_x, s.target_y)
                })
                .collect()
        };
        BubbleDropPlan { target_x: snap.target_x, target_y: snap.target_y, should_dismiss: snap.should_dismiss, displaced_siblings }
    }

    /// Computes edge tuck/untuck transition coordinates and sibling Y displacements
    pub fn compute_edge_transition_plan(
        account_index: u32,
        cur_x: i32,
        cur_y: i32,
        is_untucked: bool,
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        all_bubbles: &[BubbleRect],
    ) -> Option<HoverTransitionPlan> {
        if config.is_free_position {
            return None;
        }
        let target_pos = Self::calculate_hover_pos(screen, config, cur_x, cur_y, is_untucked);
        let mid_x = screen.x + screen.width / 2;
        let bubble_w = (config.size * screen.scale_factor) as i32;
        let is_left_edge = cur_x + bubble_w / 2 < mid_x;
        let slots = CollisionResolver::resolve_mutual_column_layout(account_index, cur_y, is_left_edge, screen, config, all_bubbles);
        let target_y = slots.iter().find(|s| s.account_index == account_index).map(|s| s.target_y).unwrap_or(target_pos.y);
        let displaced_siblings = slots
            .into_iter()
            .filter(|s| s.account_index != account_index)
            .map(|s| {
                let sib_x = all_bubbles.iter().find(|b| b.account_index == s.account_index).map(|b| b.x).unwrap_or(target_pos.x);
                (s.account_index, sib_x, s.target_y)
            })
            .collect();
        Some(HoverTransitionPlan { target_x: target_pos.x, target_y, displaced_siblings })
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

    #[test]
    fn test_can_auto_tuck_blocks_when_alert_active() {
        let idle = BubbleInteractionFlags::default();
        assert!(BubbleMovementEngine::can_auto_tuck(idle));
        let alerting = BubbleInteractionFlags { is_alert_active: true, ..Default::default() };
        assert!(!BubbleMovementEngine::can_auto_tuck(alerting));
        assert!(BubbleMovementEngine::should_stay_untucked(alerting));
        let config = BubblePhysicsConfig::default();
        assert_eq!(BubbleMovementEngine::effective_edge_ratio(&config, true), config.appear_ratio);
        assert_eq!(BubbleMovementEngine::effective_edge_ratio(&config, false), config.tuck_ratio);
    }

    #[test]
    fn test_compute_drop_plan_preserves_untucked_sibling_x() {
        let screen = ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let config = BubblePhysicsConfig::default();
        // Account 1 is alerting and untucked at left edge x=0; dragging account 2 near it must not tuck account 1
        let existing = vec![BubbleRect { account_index: 1, x: 0, y: 300, width: 56, height: 56 }];
        let plan = BubbleMovementEngine::compute_drop_plan(2, 40, 300, false, &screen, &config, &existing);
        assert!(!plan.should_dismiss);
        assert_eq!(plan.target_x, -42);
        assert_eq!(plan.displaced_siblings.len(), 1);
        assert_eq!(plan.displaced_siblings[0].0, 1);
        assert_eq!(plan.displaced_siblings[0].1, 0);
    }

    #[test]
    fn test_compute_drop_plan_keeps_alerting_moving_bubble_untucked() {
        let screen = ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let config = BubblePhysicsConfig::default();
        let existing = vec![BubbleRect { account_index: 2, x: -42, y: 300, width: 56, height: 56 }];
        let plan = BubbleMovementEngine::compute_drop_plan(1, 40, 300, true, &screen, &config, &existing);
        assert!(!plan.should_dismiss);
        assert_eq!(plan.target_x, 0);
        assert_eq!(plan.displaced_siblings.len(), 1);
        assert_eq!(plan.displaced_siblings[0].0, 2);
        assert_eq!(plan.displaced_siblings[0].1, -42);
    }
}
