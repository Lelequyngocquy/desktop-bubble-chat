use super::config::{BubblePhysicsConfig, ScreenMetrics};

/// Axis-aligned bounding box (AABB) of a bubble on screen
#[derive(Debug, Clone, Copy)]
pub struct BubbleRect {
    pub account_index: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct CollisionResolver;

impl CollisionResolver {
    /// Checks if two vertical intervals overlap with a given gap
    pub fn check_vertical_overlap(y1: i32, h1: i32, y2: i32, h2: i32, gap: i32) -> bool {
        let bottom1 = y1 + h1;
        let bottom2 = y2 + h2;
        !(bottom1 + gap <= y2 || bottom2 + gap <= y1)
    }

    /// Resolves vertical collision between bubbles along the same screen edge
    pub fn resolve_multi_bubble_collision(
        moving_index: u32,
        initial_y: i32,
        is_left_edge: bool,
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        all_bubbles: &[BubbleRect],
    ) -> i32 {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let gap = (config.gap * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
        let mid_x = screen.x + screen.width / 2;
        let mut same_edge_obstacles: Vec<BubbleRect> = all_bubbles
            .iter()
            .filter(|b| {
                if b.account_index == moving_index {
                    return false;
                }
                let b_is_left = b.x + b.width / 2 < mid_x;
                b_is_left == is_left_edge
            })
            .cloned()
            .collect();
        if same_edge_obstacles.is_empty() {
            return initial_y.clamp(min_y, max_y);
        }
        same_edge_obstacles.sort_by_key(|b| b.y);
        let mut candidate_y = initial_y.clamp(min_y, max_y);
        for _ in 0..same_edge_obstacles.len() {
            let mut has_overlap = false;
            for obs in &same_edge_obstacles {
                if Self::check_vertical_overlap(candidate_y, bubble_size, obs.y, obs.height, gap) {
                    has_overlap = true;
                    let obs_center = obs.y + obs.height / 2;
                    let cand_center = candidate_y + bubble_size / 2;
                    if cand_center >= obs_center {
                        let try_down = obs.y + obs.height + gap;
                        if try_down <= max_y {
                            candidate_y = try_down;
                        } else {
                            candidate_y = (obs.y - bubble_size - gap).max(min_y);
                        }
                    } else {
                        let try_up = obs.y - bubble_size - gap;
                        if try_up >= min_y {
                            candidate_y = try_up;
                        } else {
                            candidate_y = (obs.y + obs.height + gap).min(max_y);
                        }
                    }
                }
            }
            if !has_overlap {
                break;
            }
        }
        candidate_y.clamp(min_y, max_y)
    }

    /// Finds next available vertical slot for newly spawned bubble
    pub fn find_free_stacked_slot(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        existing_bubbles: &[BubbleRect],
        is_left_edge: bool,
    ) -> i32 {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let gap = (config.gap * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
        let default_y = (screen.y + screen.height / 3).clamp(min_y, max_y);
        let mid_x = screen.x + screen.width / 2;
        let mut same_edge_bubbles: Vec<&BubbleRect> = existing_bubbles
            .iter()
            .filter(|b| {
                if is_left_edge {
                    b.x + b.width / 2 < mid_x
                } else {
                    b.x + b.width / 2 >= mid_x
                }
            })
            .collect();
        if same_edge_bubbles.is_empty() {
            return default_y;
        }
        same_edge_bubbles.sort_by_key(|b| b.y);
        let last_bubble = same_edge_bubbles.last().unwrap();
        let candidate_below = last_bubble.y + last_bubble.height + gap;
        if candidate_below <= max_y {
            return candidate_below;
        }
        let first_bubble = same_edge_bubbles.first().unwrap();
        let candidate_above = first_bubble.y - bubble_size - gap;
        if candidate_above >= min_y {
            return candidate_above;
        }
        for window in same_edge_bubbles.windows(2) {
            let top_b = window[0];
            let bottom_b = window[1];
            let available_space = bottom_b.y - (top_b.y + top_b.height);
            if available_space >= bubble_size + gap * 2 {
                return top_b.y + top_b.height + gap;
            }
        }
        default_y
    }
}
