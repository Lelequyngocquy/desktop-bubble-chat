use crate::movement::{BubblePhysicsConfig, BubbleRect, ScreenMetrics};
use super::model::{DockActionId, DockDirection, DockTooltipSide, DOCK_ACTIONS};

pub const DOCK_PAD_X_LOGICAL: f64 = 10.0;
pub const DOCK_PAD_Y_LOGICAL: f64 = 10.0;
pub const DOCK_HIDE_EXTRA_GAP_LOGICAL: f64 = 4.0;
pub const DOCK_HOVER_CORRIDOR_EXTRA_X_LOGICAL: f64 = 6.0;
pub const DOCK_TOOLTIP_WING_LOGICAL: f64 = 172.0;
pub const DOCK_TOOLTIP_GAP_LOGICAL: f64 = 8.0;
pub const DOCK_TOOLTIP_HEIGHT_LOGICAL: f64 = 24.0;
pub const DOCK_TOOLTIP_PAD_X_LOGICAL: f64 = 9.0;

/// Physical-pixel metrics scaled cleanly from compact logical sizing presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockCompactMetrics {
    pub diameter_px: f32,
    pub anchor_gap_px: f32,
    pub item_gap_px: f32,
    pub hide_extra_gap_px: f32,
    pub pad_x_px: f32,
    pub pad_y_px: f32,
    pub stack_height_px: f32,
}

/// Computes compact physical dimensions for Sm (28px), Md (30px), and Lg (32px) bubbles
pub fn compact_metrics_for_bubble(bubble_size_px: u32, scale_factor: f64) -> DockCompactMetrics {
    let scale = if scale_factor.is_finite() && scale_factor > 0.25 { scale_factor } else { 1.0 };
    let logical_bubble = bubble_size_px as f64 / scale;
    let (diam_l, anchor_l, gap_l) = if logical_bubble <= 50.0 {
        (28.0, 7.0, 5.0)
    } else if logical_bubble >= 62.0 {
        (32.0, 8.0, 6.0)
    } else {
        (30.0, 8.0, 6.0)
    };
    let count = DOCK_ACTIONS.len() as f64;
    let extra_gaps = DOCK_ACTIONS.iter().filter(|a| a.has_extra_separator_before()).count() as f64;
    let stack_l = count * diam_l + (count - 1.0) * gap_l + extra_gaps * DOCK_HIDE_EXTRA_GAP_LOGICAL;
    DockCompactMetrics {
        diameter_px: (diam_l * scale) as f32,
        anchor_gap_px: (anchor_l * scale) as f32,
        item_gap_px: (gap_l * scale) as f32,
        hide_extra_gap_px: (DOCK_HIDE_EXTRA_GAP_LOGICAL * scale) as f32,
        pad_x_px: (DOCK_PAD_X_LOGICAL * scale) as f32,
        pad_y_px: (DOCK_PAD_Y_LOGICAL * scale) as f32,
        stack_height_px: (stack_l * scale) as f32,
    }
}

/// Local-canvas coordinates of an individual circular droplet
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockButtonSlot {
    pub action: DockActionId,
    pub local_cx: f32,
    pub local_cy: f32,
    pub radius: f32,
}

/// Physical screen-space rectangle used for Hover Bridge hit containment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    /// Returns true if physical screen coordinates fall inside this rectangle
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Local-canvas floating tooltip dialog pill geometry and opacity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockTooltipPillRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub alpha: f32,
}

/// Complete physical geometry for positioning, rendering, and hit-testing ActionDockWindow
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDockLayout {
    pub window_x: i32,
    pub window_y: i32,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub direction: DockDirection,
    pub tooltip_side: DockTooltipSide,
    pub diameter_px: f32,
    pub scale_factor: f64,
    pub slots: Vec<DockButtonSlot>,
    pub hover_corridor: ScreenRect,
    pub stack_top_y: i32,
    pub stack_bottom_y: i32,
}

/// Computes layout for an explicit vertical expansion direction with inward tooltip wing
pub fn compute_action_dock_layout_with_direction(
    bubble_x: i32,
    bubble_y: i32,
    bubble_size_px: u32,
    screen: &ScreenMetrics,
    margin_logical: f64,
    taskbar_margin_logical: f64,
    direction: DockDirection,
) -> ActionDockLayout {
    let scale = if screen.scale_factor.is_finite() && screen.scale_factor > 0.25 { screen.scale_factor } else { 1.0 };
    let m = compact_metrics_for_bubble(bubble_size_px, scale);
    let wing_px = (DOCK_TOOLTIP_WING_LOGICAL * scale).ceil() as f32;
    let canvas_w = (m.pad_x_px + m.diameter_px + wing_px).ceil() as u32;
    let canvas_h = (m.stack_height_px + 2.0 * m.pad_y_px).ceil() as u32;
    let margin_px = (margin_logical * scale).round() as i32;
    let taskbar_px = (taskbar_margin_logical * scale).round() as i32;
    let work_top = screen.y + margin_px;
    let work_bottom = (screen.y + screen.height - taskbar_px).max(work_top + m.stack_height_px.ceil() as i32);
    let bubble_cx = bubble_x + (bubble_size_px as i32 / 2);
    let screen_mid_x = screen.x + screen.width / 2;
    let tooltip_side = if bubble_cx < screen_mid_x { DockTooltipSide::Right } else { DockTooltipSide::Left };
    let edge_offset = (m.diameter_px * 0.5 + m.pad_x_px).round() as i32;
    let raw_win_x = match tooltip_side {
        DockTooltipSide::Right => bubble_cx - edge_offset,
        DockTooltipSide::Left => bubble_cx + edge_offset - canvas_w as i32,
    };
    let min_win_x = screen.x;
    let max_win_x = (screen.x + screen.width - canvas_w as i32).max(min_win_x);
    let window_x = raw_win_x.clamp(min_win_x, max_win_x);
    let stack_h_i32 = m.stack_height_px.ceil() as i32;
    let anchor_gap_i32 = m.anchor_gap_px.round() as i32;
    let raw_stack_top = match direction {
        DockDirection::Down => bubble_y + bubble_size_px as i32 + anchor_gap_i32,
        DockDirection::Up => bubble_y - anchor_gap_i32 - stack_h_i32,
    };
    let max_stack_top = (work_bottom - stack_h_i32).max(work_top);
    let stack_top_y = raw_stack_top.clamp(work_top, max_stack_top);
    let stack_bottom_y = stack_top_y + stack_h_i32;
    let window_y = stack_top_y - m.pad_y_px.round() as i32;
    let local_cx = (bubble_cx - window_x) as f32;
    let radius = m.diameter_px * 0.5;
    let mut slots = Vec::with_capacity(DOCK_ACTIONS.len());
    let mut cursor_dist = radius;
    for (idx, &action) in DOCK_ACTIONS.iter().enumerate() {
        if idx > 0 {
            cursor_dist += m.diameter_px + m.item_gap_px;
            if action.has_extra_separator_before() {
                cursor_dist += m.hide_extra_gap_px;
            }
        }
        let local_cy = match direction {
            DockDirection::Down => m.pad_y_px + cursor_dist,
            DockDirection::Up => m.pad_y_px + m.stack_height_px - cursor_dist,
        };
        slots.push(DockButtonSlot { action, local_cx, local_cy, radius });
    }
    let corridor_w = (m.diameter_px as f64 + DOCK_HOVER_CORRIDOR_EXTRA_X_LOGICAL * scale).round() as i32;
    let corridor_x = bubble_cx - corridor_w / 2;
    let (corridor_top, corridor_bottom) = match direction {
        DockDirection::Down => (bubble_y + bubble_size_px as i32 - 2, stack_bottom_y + 2),
        DockDirection::Up => (stack_top_y - 2, bubble_y + 2),
    };
    let hover_corridor = ScreenRect {
        x: corridor_x,
        y: corridor_top.min(corridor_bottom),
        width: corridor_w,
        height: (corridor_bottom - corridor_top).abs().max(1),
    };
    ActionDockLayout {
        window_x,
        window_y,
        canvas_width: canvas_w,
        canvas_height: canvas_h,
        direction,
        tooltip_side,
        diameter_px: m.diameter_px,
        scale_factor: scale,
        slots,
        hover_corridor,
        stack_top_y,
        stack_bottom_y,
    }
}

/// Computes local-canvas tooltip dialog pill rectangle beside a hovered droplet without clipping borders
pub fn compute_tooltip_pill_rect(
    layout: &ActionDockLayout,
    slot: &DockButtonSlot,
    text_width_px: f32,
    hover_t: f32,
    droplet_alpha: f32,
) -> Option<DockTooltipPillRect> {
    let alpha = (hover_t.clamp(0.0, 1.0) * droplet_alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    if alpha <= 0.01 {
        return None;
    }
    let scale = layout.scale_factor as f32;
    let gap = (DOCK_TOOLTIP_GAP_LOGICAL as f32) * scale;
    let pad_x = (DOCK_TOOLTIP_PAD_X_LOGICAL as f32) * scale;
    let outer_margin = 6.0 * scale;
    let pill_h = (DOCK_TOOLTIP_HEIGHT_LOGICAL as f32) * scale;
    let canvas_w = layout.canvas_width as f32;
    let canvas_h = layout.canvas_height as f32;
    let slide = (1.0 - hover_t.clamp(0.0, 1.0)) * 4.0 * scale;
    let avail_w = match layout.tooltip_side {
        DockTooltipSide::Right => (canvas_w - (slot.local_cx + slot.radius + gap) - outer_margin).max(24.0),
        DockTooltipSide::Left => ((slot.local_cx - slot.radius - gap) - outer_margin).max(24.0),
    };
    let desired_w = (text_width_px + 2.0 * pad_x).max(40.0 * scale);
    let pill_w = desired_w.min(avail_w);
    let raw_x = match layout.tooltip_side {
        DockTooltipSide::Right => slot.local_cx + slot.radius + gap - slide,
        DockTooltipSide::Left => slot.local_cx - slot.radius - gap - pill_w + slide,
    };
    let max_x = (canvas_w - pill_w - outer_margin).max(outer_margin);
    let x = raw_x.clamp(outer_margin, max_x);
    let min_y = 4.0 * scale;
    let max_y = (canvas_h - pill_h - 4.0 * scale).max(min_y);
    let y = (slot.local_cy - pill_h * 0.5).clamp(min_y, max_y);
    Some(DockTooltipPillRect { x, y, width: pill_w, height: pill_h, alpha })
}

/// Chooses Down by default or flips Up when bubble sits near the screen bottom
pub fn compute_action_dock_layout(
    bubble_x: i32,
    bubble_y: i32,
    bubble_size_px: u32,
    screen: &ScreenMetrics,
    margin_logical: f64,
    taskbar_margin_logical: f64,
) -> ActionDockLayout {
    let scale = if screen.scale_factor.is_finite() && screen.scale_factor > 0.25 { screen.scale_factor } else { 1.0 };
    let m = compact_metrics_for_bubble(bubble_size_px, scale);
    let margin_px = (margin_logical * scale).round() as i32;
    let taskbar_px = (taskbar_margin_logical * scale).round() as i32;
    let work_top = screen.y + margin_px;
    let work_bottom = screen.y + screen.height - taskbar_px;
    let needed = m.anchor_gap_px.round() as i32 + m.stack_height_px.ceil() as i32;
    let space_below = work_bottom - (bubble_y + bubble_size_px as i32);
    let space_above = bubble_y - work_top;
    let direction = if space_below >= needed || space_below >= space_above {
        DockDirection::Down
    } else {
        DockDirection::Up
    };
    compute_action_dock_layout_with_direction(bubble_x, bubble_y, bubble_size_px, screen, margin_logical, taskbar_margin_logical, direction)
}

/// Evaluates strict circular hit-test against individual droplets in local canvas coordinates
pub fn hit_test_dock_droplet(
    local_x: f32,
    local_y: f32,
    layout: &ActionDockLayout,
) -> Option<DockActionId> {
    let pad = (1.5 * layout.scale_factor as f32).clamp(1.0, 3.0);
    for slot in &layout.slots {
        let dx = local_x - slot.local_cx;
        let dy = local_y - slot.local_cy;
        let hit_r = slot.radius + pad;
        if dx * dx + dy * dy <= hit_r * hit_r {
            return Some(slot.action);
        }
    }
    None
}

/// Evaluates strict circular droplet hit-test from physical screen coordinates
pub fn hit_test_dock_screen_point(
    screen_x: i32,
    screen_y: i32,
    layout: &ActionDockLayout,
) -> Option<DockActionId> {
    let local_x = (screen_x - layout.window_x) as f32;
    let local_y = (screen_y - layout.window_y) as f32;
    hit_test_dock_droplet(local_x, local_y, layout)
}

/// Evaluates seamless vertical Hover Bridge corridor connecting bubble and all 6 droplets
pub fn is_point_in_dock_hover_corridor(
    screen_x: i32,
    screen_y: i32,
    layout: &ActionDockLayout,
) -> bool {
    layout.hover_corridor.contains_point(screen_x, screen_y)
}

/// Temporary displacement record for a sibling bubble pushed aside while dock is open
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SiblingPushOffset {
    pub account_index: u32,
    pub original_y: i32,
    pub target_y: i32,
}

/// Combined layout and reversible sibling displacement plan for opening ActionDock
#[derive(Debug, Clone, PartialEq)]
pub struct DockPlacementPlan {
    pub layout: ActionDockLayout,
    pub sibling_pushes: Vec<SiblingPushOffset>,
}

/// Simulates pushing same-column sibling bubbles out of the dock span, returning (pushes, overflowed)
fn plan_sibling_pushes_for_layout(
    active: BubbleRect,
    layout: &ActionDockLayout,
    screen: &ScreenMetrics,
    config: &BubblePhysicsConfig,
    all_bubbles: &[BubbleRect],
) -> (Vec<SiblingPushOffset>, bool) {
    let scale = layout.scale_factor;
    let gap_px = (config.gap * scale).round() as i32;
    let margin_px = (config.margin * scale).round() as i32;
    let taskbar_px = (config.taskbar_margin * scale).round() as i32;
    let work_top = screen.y + margin_px;
    let work_bottom = screen.y + screen.height - taskbar_px;
    let active_cx = active.x + active.width / 2;
    let col_threshold = active.width.max(24);
    let mut candidates: Vec<&BubbleRect> = all_bubbles
        .iter()
        .filter(|b| b.account_index != active.account_index)
        .filter(|b| {
            let sib_cx = b.x + b.width / 2;
            (sib_cx - active_cx).abs() < col_threshold
        })
        .filter(|b| match layout.direction {
            DockDirection::Down => b.y >= active.y,
            DockDirection::Up => b.y <= active.y,
        })
        .collect();
    match layout.direction {
        DockDirection::Down => candidates.sort_by_key(|b| b.y),
        DockDirection::Up => candidates.sort_by_key(|a| std::cmp::Reverse(a.y)),
    }
    let mut pushes = Vec::new();
    let mut overflowed = false;
    match layout.direction {
        DockDirection::Down => {
            let mut min_next_y = layout.stack_bottom_y + gap_px;
            for sib in candidates {
                if sib.y < min_next_y {
                    let max_allowed_y = (work_bottom - sib.height).max(work_top);
                    if min_next_y > max_allowed_y {
                        overflowed = true;
                    }
                    let clamped_y = min_next_y.min(max_allowed_y);
                    pushes.push(SiblingPushOffset { account_index: sib.account_index, original_y: sib.y, target_y: clamped_y });
                    min_next_y = clamped_y + sib.height + gap_px;
                } else {
                    min_next_y = sib.y + sib.height + gap_px;
                }
            }
        }
        DockDirection::Up => {
            let mut max_next_bottom = layout.stack_top_y - gap_px;
            for sib in candidates {
                let sib_bottom = sib.y + sib.height;
                if sib_bottom > max_next_bottom {
                    let desired_y = max_next_bottom - sib.height;
                    if desired_y < work_top {
                        overflowed = true;
                    }
                    let clamped_y = desired_y.max(work_top);
                    pushes.push(SiblingPushOffset { account_index: sib.account_index, original_y: sib.y, target_y: clamped_y });
                    max_next_bottom = clamped_y - gap_px;
                } else {
                    max_next_bottom = sib.y - gap_px;
                }
            }
        }
    }
    (pushes, overflowed)
}

/// Computes dock layout and reversible sibling pushes, flipping Up automatically when Down overflows
pub fn compute_dock_placement_plan(
    active_account: u32,
    bubble_x: i32,
    bubble_y: i32,
    bubble_size_px: u32,
    screen: &ScreenMetrics,
    config: &BubblePhysicsConfig,
    all_bubbles: &[BubbleRect],
) -> DockPlacementPlan {
    let active_rect = BubbleRect {
        account_index: active_account,
        x: bubble_x,
        y: bubble_y,
        width: bubble_size_px as i32,
        height: bubble_size_px as i32,
    };
    let primary = compute_action_dock_layout(bubble_x, bubble_y, bubble_size_px, screen, config.margin, config.taskbar_margin);
    let (primary_pushes, primary_overflow) = plan_sibling_pushes_for_layout(
        active_rect,
        &primary,
        screen,
        config,
        all_bubbles,
    );
    if !primary_overflow {
        return DockPlacementPlan { layout: primary, sibling_pushes: primary_pushes };
    }
    let alt_dir = match primary.direction {
        DockDirection::Down => DockDirection::Up,
        DockDirection::Up => DockDirection::Down,
    };
    let scale = primary.scale_factor;
    let margin_px = (config.margin * scale).round() as i32;
    let taskbar_px = (config.taskbar_margin * scale).round() as i32;
    let m = compact_metrics_for_bubble(bubble_size_px, scale);
    let needed = m.anchor_gap_px.round() as i32 + m.stack_height_px.ceil() as i32;
    let alt_has_room = match alt_dir {
        DockDirection::Up => (bubble_y - (screen.y + margin_px)) >= needed,
        DockDirection::Down => ((screen.y + screen.height - taskbar_px) - (bubble_y + bubble_size_px as i32)) >= needed,
    };
    if alt_has_room {
        let alt_layout = compute_action_dock_layout_with_direction(
            bubble_x,
            bubble_y,
            bubble_size_px,
            screen,
            config.margin,
            config.taskbar_margin,
            alt_dir,
        );
        let (alt_pushes, alt_overflow) = plan_sibling_pushes_for_layout(
            active_rect,
            &alt_layout,
            screen,
            config,
            all_bubbles,
        );
        if !alt_overflow {
            return DockPlacementPlan { layout: alt_layout, sibling_pushes: alt_pushes };
        }
    }
    DockPlacementPlan { layout: primary, sibling_pushes: primary_pushes }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_screen(scale: f64) -> ScreenMetrics {
        ScreenMetrics { x: 0, y: 0, width: (1920.0 * scale) as i32, height: (1080.0 * scale) as i32, scale_factor: scale }
    }

    #[test]
    fn test_compact_metrics_across_three_sizes_and_hidpi() {
        let sm = compact_metrics_for_bubble(48, 1.0);
        let md = compact_metrics_for_bubble(56, 1.0);
        let lg = compact_metrics_for_bubble(64, 1.0);
        assert_eq!(sm.diameter_px, 28.0);
        assert_eq!(md.diameter_px, 30.0);
        assert_eq!(lg.diameter_px, 32.0);
        assert_eq!(sm.stack_height_px, 164.0);
        assert_eq!(md.stack_height_px, 178.0);
        assert_eq!(lg.stack_height_px, 188.0);
        let md_2x = compact_metrics_for_bubble(112, 2.0);
        assert_eq!(md_2x.diameter_px, 60.0);
        assert_eq!(md_2x.stack_height_px, 356.0);
    }

    #[test]
    fn test_layout_flips_up_near_bottom_and_flips_tooltip_side_per_edge() {
        let screen = test_screen(1.0);
        let top_layout = compute_action_dock_layout(12, 120, 56, &screen, 12.0, 60.0);
        assert_eq!(top_layout.direction, DockDirection::Down);
        assert_eq!(top_layout.tooltip_side, DockTooltipSide::Right);
        assert!(top_layout.slots[0].local_cy < top_layout.slots[4].local_cy);
        let bottom_layout = compute_action_dock_layout(1852, 920, 56, &screen, 12.0, 60.0);
        assert_eq!(bottom_layout.direction, DockDirection::Up);
        assert_eq!(bottom_layout.tooltip_side, DockTooltipSide::Left);
        assert_eq!(bottom_layout.slots[0].action, DockActionId::Pin);
        assert_eq!(bottom_layout.slots[4].action, DockActionId::HideBubble);
        assert!(bottom_layout.slots[0].local_cy > bottom_layout.slots[4].local_cy);
        let pill = compute_tooltip_pill_rect(&bottom_layout, &bottom_layout.slots[0], 96.0, 1.0, 1.0).expect("pill");
        assert!(pill.x >= 4.0);
        assert!(pill.x + pill.width < bottom_layout.slots[0].local_cx - bottom_layout.slots[0].radius);
    }

    #[test]
    fn test_droplet_hit_test_vs_hover_bridge_corridor() {
        let screen = test_screen(1.0);
        let layout = compute_action_dock_layout(20, 140, 56, &screen, 12.0, 60.0);
        let pin = layout.slots[0];
        let resize = layout.slots[1];
        assert_eq!(hit_test_dock_droplet(pin.local_cx, pin.local_cy, &layout), Some(DockActionId::Pin));
        assert_eq!(hit_test_dock_droplet(resize.local_cx, resize.local_cy, &layout), Some(DockActionId::AllowResize));
        let gap_y = (pin.local_cy + resize.local_cy) * 0.5;
        assert_eq!(hit_test_dock_droplet(pin.local_cx, gap_y, &layout), None);
        let bridge_screen_x = layout.window_x + pin.local_cx.round() as i32;
        let anchor_gap_screen_y = 140 + 56 + 4;
        let between_btn_screen_y = layout.window_y + gap_y.round() as i32;
        assert!(is_point_in_dock_hover_corridor(bridge_screen_x, anchor_gap_screen_y, &layout));
        assert!(is_point_in_dock_hover_corridor(bridge_screen_x, between_btn_screen_y, &layout));
        assert!(!is_point_in_dock_hover_corridor(bridge_screen_x + 80, between_btn_screen_y, &layout));
    }

    #[test]
    fn test_sibling_push_clears_dock_and_flips_when_bottom_crowded() {
        let screen = test_screen(1.0);
        let config = BubblePhysicsConfig::default();
        let siblings = vec![
            BubbleRect { account_index: 1, x: 12, y: 120, width: 56, height: 56 },
            BubbleRect { account_index: 2, x: 12, y: 188, width: 56, height: 56 },
            BubbleRect { account_index: 3, x: 12, y: 256, width: 56, height: 56 },
        ];
        let plan = compute_dock_placement_plan(1, 12, 120, 56, &screen, &config, &siblings);
        assert_eq!(plan.layout.direction, DockDirection::Down);
        assert_eq!(plan.sibling_pushes.len(), 2);
        assert_eq!(plan.sibling_pushes[0].account_index, 2);
        assert_eq!(plan.sibling_pushes[0].original_y, 188);
        assert!(plan.sibling_pushes[0].target_y >= plan.layout.stack_bottom_y + 12);
        assert!(plan.sibling_pushes[1].target_y >= plan.sibling_pushes[0].target_y + 56 + 12);
        let crowded_bottom = vec![
            BubbleRect { account_index: 1, x: 12, y: 680, width: 56, height: 56 },
            BubbleRect { account_index: 2, x: 12, y: 748, width: 56, height: 56 },
            BubbleRect { account_index: 3, x: 12, y: 816, width: 56, height: 56 },
            BubbleRect { account_index: 4, x: 12, y: 884, width: 56, height: 56 },
        ];
        let flip_plan = compute_dock_placement_plan(1, 12, 680, 56, &screen, &config, &crowded_bottom);
        assert_eq!(flip_plan.layout.direction, DockDirection::Up);
        assert!(flip_plan.sibling_pushes.is_empty());
    }
}
