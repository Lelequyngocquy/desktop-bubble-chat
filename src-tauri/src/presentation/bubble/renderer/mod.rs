pub mod badge;
pub mod icons;
pub mod theme;

pub use badge::*;
pub use icons::*;
pub use theme::*;

use tiny_skia::*;

#[derive(Debug, Clone)]
pub struct BubbleRenderOptions {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub bubble_size: u32,
    pub account_index: u32,
    pub is_hovered: bool,
    pub is_left_edge: bool,
    pub unread_count: u32,
    pub has_alert_ring: bool,
    pub beat_scale: f32,
    pub ripple_progress: f32,
    pub ripple_alpha: u8,
}

impl Default for BubbleRenderOptions {
    fn default() -> Self {
        let bubble_size = 56;
        let canvas_width = bubble_size + 2 * (crate::operating_systems::current::bubble::window::BUBBLE_CANVAS_PAD_X as u32);
        let canvas_height = bubble_size + 2 * (crate::operating_systems::current::bubble::window::BUBBLE_CANVAS_PAD_Y as u32);
        Self {
            canvas_width,
            canvas_height,
            bubble_size,
            account_index: 1,
            is_hovered: false,
            is_left_edge: true,
            unread_count: 0,
            has_alert_ring: false,
            beat_scale: 1.0,
            ripple_progress: 0.0,
            ripple_alpha: 0,
        }
    }
}

/// Evaluates top-down local pixel hit-test strictly on bubble circle and unread badge (excluding outer ripple)
pub fn is_point_in_bubble_or_badge_top_down(
    local_x: f32,
    local_y: f32,
    canvas_w: f32,
    canvas_h: f32,
    bubble_size: f32,
    unread_count: u32,
    is_left_edge: bool,
) -> bool {
    let cx = canvas_w * 0.5;
    let cy = canvas_h * 0.5;
    let hit_r = (bubble_size * 0.5).max(12.0) + 2.0;
    let dx = local_x - cx;
    let dy = local_y - cy;
    if dx * dx + dy * dy <= hit_r * hit_r {
        return true;
    }
    if unread_count > 0 {
        let base_r = (bubble_size * 0.5) - 4.5;
        let is_small = base_r <= 21.5;
        let is_large = base_r > 25.5;
        let (half_w, half_h) = if unread_count <= 9 {
            let r = if is_small { 6.8 } else if is_large { 8.8 } else { 7.8 };
            (r + 2.5, r + 2.5)
        } else if unread_count <= 99 {
            let (pw, ph) = if is_small { (17.0, 13.0) } else if is_large { (22.0, 17.0) } else { (19.5, 15.0) };
            (pw * 0.5 + 2.5, ph * 0.5 + 2.5)
        } else {
            let (pw, ph) = if is_small { (21.0, 13.0) } else if is_large { (27.0, 17.0) } else { (24.0, 15.0) };
            (pw * 0.5 + 2.5, ph * 0.5 + 2.5)
        };
        let dist_x = base_r + 3.0 + (half_w - 2.5);
        let bcx = if is_left_edge { cx + dist_x } else { cx - dist_x };
        let bcy = cy - base_r * 0.35;
        if (local_x - bcx).abs() <= half_w && (local_y - bcy).abs() <= half_h {
            return true;
        }
    }
    false
}

/// Converts premultiplied RGBA from tiny-skia to 32-bit BGRA.
pub fn rgba_to_bgra(rgba: &[u8]) -> Vec<u8> {
    let mut bgra = Vec::with_capacity(rgba.len());
    for chunk in rgba.as_chunks::<4>().0 {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];
        bgra.extend_from_slice(&[b, g, r, a]);
    }
    bgra
}

/// Renders bubble via tiny-skia into a 32-bit BGRA buffer.
pub fn render_bubble_bgra(opts: &BubbleRenderOptions) -> Option<Vec<u8>> {
    let w = opts.canvas_width;
    let h = opts.canvas_height;
    let mut pixmap = Pixmap::new(w, h)?;
    pixmap.fill(Color::TRANSPARENT);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    // Bubble body radius based on logical bubble_size (leaves 4.5px for border and shadow)
    let base_radius = (opts.bubble_size as f32 / 2.0) - 4.5;
    let radius = if opts.is_hovered {
        base_radius + 0.5
    } else {
        base_radius
    };
    let dpi_scale = (opts.bubble_size as f32 / 56.0).clamp(0.85, 2.5);
    let beat_scale = if opts.beat_scale.is_finite() && opts.beat_scale > 0.0 { opts.beat_scale } else { 1.0 };
    let group_xf = if (beat_scale - 1.0).abs() > 1e-4 {
        Transform::from_translate(cx, cy).pre_scale(beat_scale, beat_scale).pre_translate(-cx, -cy)
    } else {
        Transform::identity()
    };
    let theme = get_account_theme(opts.account_index);
    // 0. Independent expanding red ripple wave underneath bubble
    if opts.has_alert_ring && opts.ripple_alpha > 0 {
        let rp = opts.ripple_progress.clamp(0.0, 1.0);
        let ease_rp = 1.0 - (1.0 - rp) * (1.0 - rp);
        let max_expand = 9.0 * dpi_scale;
        let base_ring_w = (2.0 * dpi_scale).clamp(1.8, 4.2);
        let ripple_r = radius + 1.0 + base_ring_w * 0.5 + ease_rp * max_expand;
        let ripple_stroke_w = ((2.2 - 1.2 * rp) * dpi_scale).max(1.0);
        let mut ripple_paint = Paint::default();
        ripple_paint.set_color_rgba8(239, 68, 68, opts.ripple_alpha);
        ripple_paint.anti_alias = true;
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, ripple_r);
        if let Some(ripple_path) = pb.finish() {
            let stroke = Stroke { width: ripple_stroke_w, ..Default::default() };
            pixmap.stroke_path(&ripple_path, &ripple_paint, &stroke, Transform::identity(), None);
        }
    }
    // 1. Soft drop shadow
    {
        let mut shadow_paint = Paint::default();
        shadow_paint.set_color_rgba8(0, 0, 0, 45);
        shadow_paint.anti_alias = true;
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy + 2.5, radius + 1.0);
        if let Some(shadow_path) = pb.finish() {
            pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, group_xf, None);
        }
    }
    // 2. Main bubble body (linear gradient top-to-bottom)
    {
        let start = Point::from_xy(cx, cy - radius);
        let end = Point::from_xy(cx, cy + radius);
        let stops = vec![
            GradientStop::new(0.0, Color::from_rgba8(theme.primary.0, theme.primary.1, theme.primary.2, 255)),
            GradientStop::new(1.0, Color::from_rgba8(theme.secondary.0, theme.secondary.1, theme.secondary.2, 255)),
        ];
        let shader = LinearGradient::new(
            start,
            end,
            stops,
            SpreadMode::Pad,
            Transform::identity(),
        )?;
        let body_paint = Paint { shader, anti_alias: true, ..Default::default() };
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        if let Some(body_path) = pb.finish() {
            pixmap.fill_path(&body_path, &body_paint, FillRule::Winding, group_xf, None);
            // White border (2px)
            let mut border_paint = Paint::default();
            border_paint.set_color_rgba8(255, 255, 255, 245);
            border_paint.anti_alias = true;
            let stroke = Stroke { width: 2.0, ..Default::default() };
            pixmap.stroke_path(&body_path, &border_paint, &stroke, group_xf, None);
        }
    }
    // 2.5. Base red alert ring hugging outside of white border, locked to group transform
    if opts.has_alert_ring {
        let ring_w = (2.0 * dpi_scale).clamp(1.8, 4.2);
        let ring_r = radius + 1.0 + ring_w * 0.5;
        let mut ring_paint = Paint::default();
        ring_paint.set_color_rgba8(232, 17, 35, 245);
        ring_paint.anti_alias = true;
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, ring_r);
        if let Some(ring_path) = pb.finish() {
            let stroke = Stroke { width: ring_w, ..Default::default() };
            pixmap.stroke_path(&ring_path, &ring_paint, &stroke, group_xf, None);
        }
    }
    // 3. White Messenger vector icon in center
    draw_messenger_icon(&mut pixmap, cx, cy, radius, group_xf);
    // 4. Pointed unread badge locked to bubble via unified group transform
    draw_unread_badge(&mut pixmap, cx, cy, base_radius, opts.unread_count, opts.is_left_edge, group_xf);
    Some(rgba_to_bgra(pixmap.data()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_bubble_dimensions_and_transparency() {
        let opts = BubbleRenderOptions {
            canvas_width: 124,
            canvas_height: 88,
            bubble_size: 64,
            account_index: 1,
            is_hovered: false,
            is_left_edge: true,
            unread_count: 3,
            ..Default::default()
        };
        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed");
        assert_eq!(bgra.len(), 124 * 88 * 4, "Buffer size must match 124x88x4 bytes");
        // 1. Top-left corner (0, 0) must be fully transparent (Alpha = 0)
        let top_left_alpha = bgra[3];
        assert_eq!(top_left_alpha, 0, "Corner (0, 0) must be 100% transparent alpha");
        // 2. Center (62, 44) must be opaque (Alpha > 200)
        let center_idx = (44 * 124 + 62) * 4;
        let center_alpha = bgra[center_idx + 3];
        assert!(center_alpha > 200, "Center (62, 44) must be solid alpha, got {}", center_alpha);
    }

    #[test]
    fn test_render_bubble_unread_badge_single_digit() {
        let opts = BubbleRenderOptions {
            canvas_width: 124,
            canvas_height: 88,
            bubble_size: 64,
            account_index: 2,
            is_hovered: false,
            is_left_edge: true,
            unread_count: 5,
            ..Default::default()
        };
        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for single digit");
        assert_eq!(bgra.len(), 124 * 88 * 4);
    }

    #[test]
    fn test_render_bubble_unread_badge_double_digit() {
        let opts = BubbleRenderOptions {
            canvas_width: 124,
            canvas_height: 88,
            bubble_size: 64,
            account_index: 3,
            is_hovered: false,
            is_left_edge: false,
            unread_count: 24,
            ..Default::default()
        };
        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for double digit");
        assert_eq!(bgra.len(), 124 * 88 * 4);
    }

    #[test]
    fn test_render_bubble_unread_badge_overflow() {
        let opts = BubbleRenderOptions {
            canvas_width: 124,
            canvas_height: 88,
            bubble_size: 64,
            account_index: 4,
            is_hovered: false,
            is_left_edge: false,
            unread_count: 120,
            ..Default::default()
        };
        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for overflow 99+");
        assert_eq!(bgra.len(), 124 * 88 * 4);
    }

    #[test]
    fn test_render_bubble_pointed_badge_left_and_right_edge() {
        let opts_left = BubbleRenderOptions {
            canvas_width: 124,
            canvas_height: 88,
            bubble_size: 64,
            account_index: 1,
            is_hovered: false,
            is_left_edge: true,
            unread_count: 9,
            ..Default::default()
        };
        let bgra_left = render_bubble_bgra(&opts_left).expect("render_bubble_bgra failed for left edge");
        assert_eq!(bgra_left.len(), 124 * 88 * 4);
        let opts_right = BubbleRenderOptions {
            canvas_width: 124,
            canvas_height: 88,
            bubble_size: 64,
            account_index: 1,
            is_hovered: false,
            is_left_edge: false,
            unread_count: 9,
            ..Default::default()
        };
        let bgra_right = render_bubble_bgra(&opts_right).expect("render_bubble_bgra failed for right edge");
        assert_eq!(bgra_right.len(), 124 * 88 * 4);
        assert_ne!(bgra_left, bgra_right, "Left-edge badge and Right-edge badge must render at different positions");
    }

    #[test]
    fn test_render_bubble_alert_ring_ripple_and_no_clipping_across_dpi() {
        for bubble_size in [48u32, 56, 64, 128] {
            let canvas_w = bubble_size + 60;
            let canvas_h = bubble_size + 48;
            let idle = BubbleRenderOptions {
                canvas_width: canvas_w,
                canvas_height: canvas_h,
                bubble_size,
                account_index: 2,
                unread_count: 3,
                ..Default::default()
            };
            let rest_alert = BubbleRenderOptions { has_alert_ring: true, ..idle.clone() };
            let peak_ripple = BubbleRenderOptions {
                has_alert_ring: true,
                beat_scale: 1.07,
                ripple_progress: 0.95,
                ripple_alpha: 120,
                ..idle.clone()
            };
            let buf_idle = render_bubble_bgra(&idle).expect("idle render");
            let buf_rest = render_bubble_bgra(&rest_alert).expect("rest alert render");
            let buf_peak = render_bubble_bgra(&peak_ripple).expect("peak ripple render");
            assert_ne!(buf_idle, buf_rest, "Base alert ring must visibly render");
            assert_ne!(buf_rest, buf_peak, "Heartbeat scale and ripple must visibly render");
            let top_mid = ((canvas_w / 2) * 4 + 3) as usize;
            let bot_mid = (((canvas_h - 1) * canvas_w + (canvas_w / 2)) * 4 + 3) as usize;
            assert_eq!(buf_peak[3], 0, "Top-left corner must stay transparent");
            assert_eq!(buf_peak[top_mid], 0, "Top edge must not clip ripple at size {}", bubble_size);
            assert_eq!(buf_peak[bot_mid], 0, "Bottom edge must not clip ripple at size {}", bubble_size);
        }
    }

    #[test]
    fn test_top_down_hit_zone_excludes_outer_ripple() {
        let w = 116.0;
        let h = 104.0;
        let sz = 56.0;
        assert!(is_point_in_bubble_or_badge_top_down(58.0, 52.0, w, h, sz, 0, true));
        assert!(!is_point_in_bubble_or_badge_top_down(58.0, 14.0, w, h, sz, 0, true));
        assert!(!is_point_in_bubble_or_badge_top_down(58.0, 14.0, w, h, sz, 5, true));
        assert!(is_point_in_bubble_or_badge_top_down(92.0, 44.0, w, h, sz, 5, true));
        assert!(!is_point_in_bubble_or_badge_top_down(92.0, 44.0, w, h, sz, 0, true));
    }
}
