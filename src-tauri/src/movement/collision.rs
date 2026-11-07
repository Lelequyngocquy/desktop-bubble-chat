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

/// Assigned target Y position for each bubble in a vertical column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnBubbleSlot {
    pub account_index: u32,
    pub target_y: i32,
}

pub struct CollisionResolver;

impl CollisionResolver {
    /// Checks if two vertical intervals overlap with a given gap
    pub fn check_vertical_overlap(y1: i32, h1: i32, y2: i32, h2: i32, gap: i32) -> bool {
        let bottom1 = y1 + h1;
        let bottom2 = y2 + h2;
        !(bottom1 + gap <= y2 || bottom2 + gap <= y1)
    }

    /// Checks if two horizontal intervals overlap with a given gap
    pub fn check_horizontal_overlap(x1: i32, w1: i32, x2: i32, w2: i32, gap: i32) -> bool {
        let right1 = x1 + w1;
        let right2 = x2 + w2;
        !(right1 + gap <= x2 || right2 + gap <= x1)
    }

    /// Checks if two bounding boxes overlap in 2D space with a given gap
    pub fn check_2d_overlap(b1: &BubbleRect, b2: &BubbleRect, gap: i32) -> bool {
        Self::check_horizontal_overlap(b1.x, b1.width, b2.x, b2.width, gap)
            && Self::check_vertical_overlap(b1.y, b1.height, b2.y, b2.height, gap)
    }

    /// Resolves mutual vertical layout for all bubbles along the same screen edge.
    /// Returns target Y for moving bubble and all displaced stationary bubbles.
    pub fn resolve_mutual_column_layout(
        moving_index: u32,
        desired_y: i32,
        is_left_edge: bool,
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        all_bubbles: &[BubbleRect],
    ) -> Vec<ColumnBubbleSlot> {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let gap = (config.gap * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
        let mid_x = screen.x + screen.width / 2;
        let min_spacing = bubble_size + gap;
        let same_edge_obstacles: Vec<BubbleRect> = all_bubbles
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
            return vec![ColumnBubbleSlot {
                account_index: moving_index,
                target_y: desired_y.clamp(min_y, max_y),
            }];
        }
        let clamped_desired_y = desired_y.clamp(min_y, max_y);
        let mut items: Vec<(u32, f64, bool)> = Vec::with_capacity(same_edge_obstacles.len() + 1);
        items.push((moving_index, clamped_desired_y as f64, true));
        for obs in &same_edge_obstacles {
            let obs_y = obs.y.clamp(min_y, max_y);
            let candidate_y = if obs_y == clamped_desired_y {
                if clamped_desired_y + min_spacing <= max_y {
                    obs_y as f64 + 0.1
                } else {
                    obs_y as f64 - 0.1
                }
            } else {
                obs_y as f64
            };
            items.push((obs.account_index, candidate_y, false));
        }
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let n = items.len();
        let mut positions: Vec<i32> = items.iter().map(|item| item.1.round() as i32).collect();
        // Pass 1: Forward downward push
        for i in 1..n {
            if positions[i] < positions[i - 1] + min_spacing {
                positions[i] = positions[i - 1] + min_spacing;
            }
        }
        // Pass 2: Upward cascade if bottom boundary exceeded
        if positions[n - 1] > max_y {
            positions[n - 1] = max_y;
            for i in (0..n - 1).rev() {
                if positions[i] > positions[i + 1] - min_spacing {
                    positions[i] = positions[i + 1] - min_spacing;
                }
            }
        }
        // Pass 3: Downward clamp if top boundary exceeded
        if positions[0] < min_y {
            positions[0] = min_y;
            for i in 1..n {
                if positions[i] < positions[i - 1] + min_spacing {
                    positions[i] = positions[i - 1] + min_spacing;
                }
            }
        }
        items
            .into_iter()
            .enumerate()
            .map(|(i, (idx, _, _))| ColumnBubbleSlot {
                account_index: idx,
                target_y: positions[i].clamp(min_y, max_y),
            })
            .collect()
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
        let layout = Self::resolve_mutual_column_layout(
            moving_index,
            initial_y,
            is_left_edge,
            screen,
            config,
            all_bubbles,
        );
        layout
            .iter()
            .find(|slot| slot.account_index == moving_index)
            .map(|slot| slot.target_y)
            .unwrap_or(initial_y)
    }

    /// Resolves 2D collision in free placement mode.
    /// Pushes the moving bubble away from any overlapping bubbles in 2D space (both X and Y axes).
    pub fn resolve_2d_free_collision(
        moving_index: u32,
        desired_x: i32,
        desired_y: i32,
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        all_bubbles: &[BubbleRect],
    ) -> (i32, i32) {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let gap = (config.gap * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let min_x = screen.x + margin;
        let max_x = screen.x + screen.width - bubble_size - margin;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
        let mut cur_x = desired_x.clamp(min_x, max_x);
        let mut cur_y = desired_y.clamp(min_y, max_y);
        let obstacles: Vec<BubbleRect> = all_bubbles.iter().filter(|b| b.account_index != moving_index).cloned().collect();
        if obstacles.is_empty() {
            return (cur_x, cur_y);
        }
        let radius = bubble_size as f64 / 2.0;
        let min_distance = (bubble_size + gap) as f64;
        // Iterative relaxation in 2D space
        for _ in 0..10 {
            let mut moved = false;
            let center_x = cur_x as f64 + radius;
            let center_y = cur_y as f64 + radius;
            for obs in &obstacles {
                let obs_center_x = obs.x as f64 + radius;
                let obs_center_y = obs.y as f64 + radius;
                let dx = center_x - obs_center_x;
                let dy = center_y - obs_center_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < min_distance * min_distance {
                    let dist = dist_sq.sqrt();
                    let (push_x, push_y) = if dist > 0.001 {
                        let overlap = min_distance - dist;
                        ((dx / dist) * overlap, (dy / dist) * overlap)
                    } else {
                        // Push towards screen center if overlapping
                        let mid_x = screen.x as f64 + screen.width as f64 / 2.0;
                        let mid_y = screen.y as f64 + screen.height as f64 / 2.0;
                        let to_center_x = mid_x - obs_center_x;
                        let to_center_y = mid_y - obs_center_y;
                        let c_dist = (to_center_x * to_center_x + to_center_y * to_center_y).sqrt();
                        if c_dist > 0.001 {
                            ((to_center_x / c_dist) * min_distance, (to_center_y / c_dist) * min_distance)
                        } else {
                            (min_distance, 0.0)
                        }
                    };
                    let new_x = (cur_x as f64 + push_x).round() as i32;
                    let new_y = (cur_y as f64 + push_y).round() as i32;
                    cur_x = new_x.clamp(min_x, max_x);
                    cur_y = new_y.clamp(min_y, max_y);
                    // Displace along perpendicular axis if boundary clamping preserved overlap
                    let post_dx = (cur_x as f64 + radius) - obs_center_x;
                    let post_dy = (cur_y as f64 + radius) - obs_center_y;
                    if post_dx * post_dx + post_dy * post_dy < min_distance * min_distance {
                        if cur_x == min_x || cur_x == max_x {
                            let sign_y = if dy >= 0.0 { 1.0 } else { -1.0 };
                            cur_y = (obs.y as f64 + sign_y * min_distance).round() as i32;
                            cur_y = cur_y.clamp(min_y, max_y);
                        } else if cur_y == min_y || cur_y == max_y {
                            let sign_x = if dx >= 0.0 { 1.0 } else { -1.0 };
                            cur_x = (obs.x as f64 + sign_x * min_distance).round() as i32;
                            cur_x = cur_x.clamp(min_x, max_x);
                        }
                    }
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        (cur_x, cur_y)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_resolution_same_y() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32; // 56
        let min_spacing = bubble_size + config.gap as i32;
        let obstacles = vec![
            BubbleRect {
                account_index: 1,
                x: 1906,
                y: 300,
                width: bubble_size,
                height: bubble_size,
            },
            BubbleRect {
                account_index: 2,
                x: 1800,
                y: 300,
                width: bubble_size,
                height: bubble_size,
            },
        ];
        let layout = CollisionResolver::resolve_mutual_column_layout(
            2,
            300,
            false,
            &screen,
            &config,
            &obstacles,
        );
        let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
        let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
        assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
    }

    #[test]
    fn test_mutual_column_layout_same_y() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32; // 56
        let min_spacing = bubble_size + config.gap as i32; // 68
        let obstacles = vec![
            BubbleRect {
                account_index: 1,
                x: 1906,
                y: 300,
                width: bubble_size,
                height: bubble_size,
            },
        ];
        let layout = CollisionResolver::resolve_mutual_column_layout(
            2,
            300,
            false,
            &screen,
            &config,
            &obstacles,
        );
        assert_eq!(layout.len(), 2);
        let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
        let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
        assert_eq!(b2.target_y, 300);
        assert_eq!(b1.target_y, 300 + min_spacing);
        assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
    }

    #[test]
    fn test_mutual_column_layout_top_boundary() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32;
        let min_spacing = bubble_size + config.gap as i32;
        let min_y = screen.y + config.margin as i32;
        let obstacles = vec![
            BubbleRect {
                account_index: 1,
                x: 1906,
                y: 15,
                width: bubble_size,
                height: bubble_size,
            },
        ];
        let layout = CollisionResolver::resolve_mutual_column_layout(
            2,
            15,
            false,
            &screen,
            &config,
            &obstacles,
        );
        assert_eq!(layout.len(), 2);
        for s in &layout {
            assert!(s.target_y >= min_y);
        }
        let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
        let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
        assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
    }

    #[test]
    fn test_mutual_column_layout_bottom_boundary() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32;
        let min_spacing = bubble_size + config.gap as i32;
        let max_y = screen.y + screen.height - bubble_size - config.taskbar_margin as i32;
        let obstacles = vec![
            BubbleRect {
                account_index: 1,
                x: 1906,
                y: max_y,
                width: bubble_size,
                height: bubble_size,
            },
        ];
        let layout = CollisionResolver::resolve_mutual_column_layout(
            2,
            max_y,
            false,
            &screen,
            &config,
            &obstacles,
        );
        assert_eq!(layout.len(), 2);
        for s in &layout {
            assert!(s.target_y <= max_y);
        }
        let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
        let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
        assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
    }

    #[test]
    fn test_mutual_column_layout_three_bubbles_cascade() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32;
        let min_spacing = bubble_size + config.gap as i32;
        let obstacles = vec![
            BubbleRect {
                account_index: 1,
                x: 1906,
                y: 300,
                width: bubble_size,
                height: bubble_size,
            },
            BubbleRect {
                account_index: 3,
                x: 1906,
                y: 368,
                width: bubble_size,
                height: bubble_size,
            },
        ];
        let layout = CollisionResolver::resolve_mutual_column_layout(
            2,
            300,
            false,
            &screen,
            &config,
            &obstacles,
        );
        assert_eq!(layout.len(), 3);
        let mut sorted_y: Vec<i32> = layout.iter().map(|s| s.target_y).collect();
        sorted_y.sort();
        for window in sorted_y.windows(2) {
            assert!(window[1] - window[0] >= min_spacing);
        }
    }

    #[test]
    fn test_resolve_2d_free_collision_avoidance() {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32;
        let min_spacing = (bubble_size + config.gap as i32) as f64;

        // Bubble 1 placed in middle of screen (X: 500, Y: 500)
        let obstacles = vec![BubbleRect {
            account_index: 1,
            x: 500,
            y: 500,
            width: bubble_size,
            height: bubble_size,
        }];

        // Moving bubble dropped exactly at the same position (X: 500, Y: 500)
        let (resolved_x, resolved_y) = CollisionResolver::resolve_2d_free_collision(
            2,
            500,
            500,
            &screen,
            &config,
            &obstacles,
        );

        // Distance after resolution must be >= min_spacing
        let dx = (resolved_x - 500) as f64;
        let dy = (resolved_y - 500) as f64;
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(dist >= min_spacing - 1.0);
    }

    #[test]
    fn test_horizontal_and_2d_overlap_detection() {
        assert!(CollisionResolver::check_horizontal_overlap(100, 56, 120, 56, 12));
        assert!(!CollisionResolver::check_horizontal_overlap(100, 56, 200, 56, 12));
        let b1 = BubbleRect { account_index: 1, x: 100, y: 100, width: 56, height: 56 };
        let b2 = BubbleRect { account_index: 2, x: 120, y: 120, width: 56, height: 56 };
        let b3 = BubbleRect { account_index: 3, x: 300, y: 100, width: 56, height: 56 };
        assert!(CollisionResolver::check_2d_overlap(&b1, &b2, 12));
        assert!(!CollisionResolver::check_2d_overlap(&b1, &b3, 12));
    }

    #[test]
    fn test_resolve_2d_free_collision_screen_boundary() {
        let screen = ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let config = BubblePhysicsConfig::default();
        let bubble_size = config.size as i32;
        let margin = config.margin as i32;
        let obstacles = vec![BubbleRect { account_index: 1, x: margin, y: margin, width: bubble_size, height: bubble_size }];
        // Drop bubble directly on the obstacle at screen top-left corner
        let (rx, ry) = CollisionResolver::resolve_2d_free_collision(2, margin, margin, &screen, &config, &obstacles);
        assert!(rx >= margin);
        assert!(ry >= margin);
        let dist = (((rx - margin).pow(2) + (ry - margin).pow(2)) as f64).sqrt();
        assert!(dist >= (bubble_size + config.gap as i32) as f64 - 1.0);
    }
}
