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
}

impl Default for BubbleRenderOptions {
    fn default() -> Self {
        let bubble_size = 56;
        let canvas_width = bubble_size + 2 * (crate::operating_systems::linux::bubble::window::BUBBLE_CANVAS_PAD_X as u32);
        let canvas_height = bubble_size + 2 * (crate::operating_systems::linux::bubble::window::BUBBLE_CANVAS_PAD_Y as u32);
        Self {
            canvas_width,
            canvas_height,
            bubble_size,
            account_index: 1,
            is_hovered: false,
            is_left_edge: true,
            unread_count: 0,
        }
    }
}

/// Converts premultiplied RGBA from tiny-skia to 32-bit ARGB/BGRA format
pub fn rgba_to_bgra(rgba: &[u8]) -> Vec<u8> {
    let mut bgra = Vec::with_capacity(rgba.len());
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];
        bgra.extend_from_slice(&[b, g, r, a]);
    }
    bgra
}

/// Renders bubble via tiny-skia into a 32-bit pixel buffer for Cairo / GTK window
pub fn render_bubble_bgra(opts: &BubbleRenderOptions) -> Option<Vec<u8>> {
    let w = opts.canvas_width;
    let h = opts.canvas_height;
    let mut pixmap = Pixmap::new(w, h)?;
    pixmap.fill(Color::TRANSPARENT);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let base_radius = (opts.bubble_size as f32 / 2.0) - 4.5;
    let radius = if opts.is_hovered { base_radius + 0.5 } else { base_radius };
    let theme = get_account_theme(opts.account_index);
    {
        let mut shadow_paint = Paint::default();
        shadow_paint.set_color_rgba8(0, 0, 0, 45);
        shadow_paint.anti_alias = true;
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy + 2.5, radius + 1.0);
        if let Some(shadow_path) = pb.finish() {
            pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, Transform::identity(), None);
        }
    }
    {
        let start = Point::from_xy(cx, cy - radius);
        let end = Point::from_xy(cx, cy + radius);
        let stops = vec![
            GradientStop::new(0.0, Color::from_rgba8(theme.primary.0, theme.primary.1, theme.primary.2, 255)),
            GradientStop::new(1.0, Color::from_rgba8(theme.secondary.0, theme.secondary.1, theme.secondary.2, 255)),
        ];
        let shader = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity())?;
        let mut body_paint = Paint::default();
        body_paint.shader = shader;
        body_paint.anti_alias = true;
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        if let Some(body_path) = pb.finish() {
            pixmap.fill_path(&body_path, &body_paint, FillRule::Winding, Transform::identity(), None);
            let mut border_paint = Paint::default();
            border_paint.set_color_rgba8(255, 255, 255, 245);
            border_paint.anti_alias = true;
            let mut stroke = Stroke::default();
            stroke.width = 2.0;
            pixmap.stroke_path(&body_path, &border_paint, &stroke, Transform::identity(), None);
        }
    }
    draw_messenger_icon(&mut pixmap, cx, cy, radius);
    draw_unread_badge(&mut pixmap, cx, cy, radius, opts.unread_count, opts.is_left_edge);
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
        };
        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed");
        assert_eq!(bgra.len(), 124 * 88 * 4);
        assert_eq!(bgra[3], 0);
        let center_idx = (44 * 124 + 62) * 4;
        assert!(bgra[center_idx + 3] > 200);
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
        };
        let bgra_right = render_bubble_bgra(&opts_right).expect("render_bubble_bgra failed for right edge");
        assert_eq!(bgra_right.len(), 124 * 88 * 4);
        assert_ne!(bgra_left, bgra_right);
    }
}
