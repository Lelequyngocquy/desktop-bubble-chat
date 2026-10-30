use super::config::{BubblePhysicsConfig, ScreenMetrics};

/// Mô tả vùng hình chữ nhật (AABB) của một bong bóng trên màn hình
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
    /// Kiểm tra xem 2 khoảng chiều cao [y1, y1 + h1] và [y2, y2 + h2] có bị va chạm (khoảng cách < gap) hay không
    pub fn check_vertical_overlap(y1: i32, h1: i32, y2: i32, h2: i32, gap: i32) -> bool {
        let bottom1 = y1 + h1;
        let bottom2 = y2 + h2;
        !(bottom1 + gap <= y2 || bottom2 + gap <= y1)
    }

    /// Giải quyết va chạm dọc giữa nhiều Bubble trên cùng một mép màn hình (Multi-body Collision Resolution)
    /// Đảm bảo sau khi giải quyết, bong bóng đang kéo (moving_index) không đè lên bất kỳ bubble nào khác,
    /// và luôn nằm an toàn trong khoảng [min_y, max_y] của màn hình.
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

        // Lọc các bubble khác đang ở cùng mép màn hình (trái hoặc phải)
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

        // Sắp xếp các vật cản theo chiều từ trên xuống dưới
        same_edge_obstacles.sort_by_key(|b| b.y);

        let mut candidate_y = initial_y.clamp(min_y, max_y);

        // Vòng lặp giải quyết va chạm lặp (Iterative Relaxation): Tối đa 5 lượt duyệt (vì có tối đa 5 bubble)
        for _ in 0..same_edge_obstacles.len() {
            let mut has_overlap = false;

            for obs in &same_edge_obstacles {
                if Self::check_vertical_overlap(candidate_y, bubble_size, obs.y, obs.height, gap) {
                    has_overlap = true;
                    // Nếu tâm của candidate_y thấp hơn hoặc bằng tâm của vật cản -> đẩy xuống dưới
                    let obs_center = obs.y + obs.height / 2;
                    let cand_center = candidate_y + bubble_size / 2;

                    if cand_center >= obs_center {
                        // Thử đẩy xuống dưới vật cản
                        let try_down = obs.y + obs.height + gap;
                        if try_down <= max_y {
                            candidate_y = try_down;
                        } else {
                            // Nếu chạm đáy màn hình, buộc phải đảo chiều đẩy lên trên
                            candidate_y = (obs.y - bubble_size - gap).max(min_y);
                        }
                    } else {
                        // Thử đẩy lên trên vật cản
                        let try_up = obs.y - bubble_size - gap;
                        if try_up >= min_y {
                            candidate_y = try_up;
                        } else {
                            // Nếu chạm đỉnh màn hình, buộc phải đẩy xuống dưới
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

    /// Tìm slot trống tiếp theo để phân bổ cho một Bubble mới sinh ra sao cho không đè lên các Bubble đã có
    pub fn find_free_stacked_slot(
        screen: &ScreenMetrics,
        config: &BubblePhysicsConfig,
        existing_bubbles: &[BubbleRect],
    ) -> i32 {
        let scale = screen.scale_factor;
        let bubble_size = (config.size * scale) as i32;
        let gap = (config.gap * scale) as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;

        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
        let default_y = (screen.y + screen.height / 3).clamp(min_y, max_y);

        // Lọc các bubble đang ở mép phải (vị trí mặc định)
        let mid_x = screen.x + screen.width / 2;
        let mut right_bubbles: Vec<&BubbleRect> = existing_bubbles
            .iter()
            .filter(|b| b.x + b.width / 2 >= mid_x)
            .collect();

        if right_bubbles.is_empty() {
            return default_y;
        }

        right_bubbles.sort_by_key(|b| b.y);

        // Thử tìm slot trống ngay dưới bubble thấp nhất
        let last_bubble = right_bubbles.last().unwrap();
        let candidate_below = last_bubble.y + last_bubble.height + gap;
        if candidate_below <= max_y {
            return candidate_below;
        }

        // Nếu phía dưới đã chạm mép taskbar, thử xếp lên trên bubble cao nhất
        let first_bubble = right_bubbles.first().unwrap();
        let candidate_above = first_bubble.y - bubble_size - gap;
        if candidate_above >= min_y {
            return candidate_above;
        }

        // Nếu cả trên và dưới đều chật, tìm khe hở giữa 2 bubble liên tiếp
        for window in right_bubbles.windows(2) {
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
