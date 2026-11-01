pub mod badge;
pub mod icons;
pub mod theme;

pub use badge::*;
pub use icons::*;
pub use theme::*;

use tiny_skia::*;

#[derive(Debug, Clone)]
pub struct BubbleRenderOptions {
    pub size: u32,
    pub account_index: u32,
    pub is_hovered: bool,
    pub has_pulse: bool,
    pub pulse_progress: f32, // Tỉ lệ hoạt ảnh sóng lan tỏa [0.0, 1.0]
    pub unread_count: u32,
}

impl Default for BubbleRenderOptions {
    fn default() -> Self {
        Self {
            size: 64,
            account_index: 1,
            is_hovered: false,
            has_pulse: false,
            pulse_progress: 0.0,
            unread_count: 0,
        }
    }
}

/// Chuyển đổi dữ liệu premultiplied RGBA của tiny-skia sang chuẩn BGRA của Windows DIB Section
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

/// Vẽ Bong bóng nổi bằng tiny-skia và xuất ra buffer 32-bit BGRA tương thích 100% với Win32 UpdateLayeredWindow
pub fn render_bubble_bgra(opts: &BubbleRenderOptions) -> Option<Vec<u8>> {
    let w = opts.size;
    let h = opts.size;
    let mut pixmap = Pixmap::new(w, h)?;
    pixmap.fill(Color::TRANSPARENT);

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    // Bán kính bóng bóng, chừa 4px cho hiệu ứng shadow và viền
    let base_radius = (w as f32 / 2.0) - 4.5;
    let radius = if opts.is_hovered {
        base_radius + 0.5
    } else {
        base_radius
    };

    let theme = get_account_theme(opts.account_index);

    // 1. Vẽ bóng đổ mềm (Soft Drop Shadow)
    {
        let mut shadow_paint = Paint::default();
        shadow_paint.set_color_rgba8(0, 0, 0, 45);
        shadow_paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy + 2.5, radius + 1.0);
        if let Some(shadow_path) = pb.finish() {
            pixmap.fill_path(
                &shadow_path,
                &shadow_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    // 2. Vẽ viền sóng lan tỏa Pulse (khi có tin nhắn mới)
    if opts.has_pulse {
        let progress = opts.pulse_progress.clamp(0.0, 1.0);
        let pulse_r = radius + 0.5 + progress * 3.5;
        let alpha = ((1.0 - progress) * 175.0) as u8;

        if alpha > 0 {
            let mut pulse_paint = Paint::default();
            pulse_paint.set_color_rgba8(theme.primary.0, theme.primary.1, theme.primary.2, alpha);
            pulse_paint.anti_alias = true;

            let mut pb = PathBuilder::new();
            pb.push_circle(cx, cy, pulse_r);
            if let Some(pulse_path) = pb.finish() {
                let mut stroke = Stroke::default();
                stroke.width = (2.2 - progress * 1.0).max(0.8);
                pixmap.stroke_path(
                    &pulse_path,
                    &pulse_paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    // 3. Vẽ thân chính Bong bóng (Linear Gradient từ trên xuống)
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

        let mut body_paint = Paint::default();
        body_paint.shader = shader;
        body_paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        if let Some(body_path) = pb.finish() {
            pixmap.fill_path(
                &body_path,
                &body_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            // Viền ngoài màu trắng (White Border 2px)
            let mut border_paint = Paint::default();
            border_paint.set_color_rgba8(255, 255, 255, 245);
            border_paint.anti_alias = true;

            let mut stroke = Stroke::default();
            stroke.width = 2.0;
            pixmap.stroke_path(
                &body_path,
                &border_paint,
                &stroke,
                Transform::identity(),
                None,
            );
        }
    }

    // 4. Vẽ biểu tượng Vector Messenger chuẩn SVG màu trắng ở chính giữa (theo MessengerIcon.tsx)
    draw_messenger_icon(&mut pixmap, cx, cy, radius);

    // 5. Vẽ huy hiệu đếm chưa đọc (Unread Badge & Text) nếu > 0
    draw_unread_badge(&mut pixmap, cx, cy, radius, opts.unread_count);

    Some(rgba_to_bgra(pixmap.data()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_bubble_dimensions_and_transparency() {
        let opts = BubbleRenderOptions {
            size: 64,
            account_index: 1,
            is_hovered: false,
            has_pulse: false,
            pulse_progress: 0.0,
            unread_count: 3,
        };

        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed");
        assert_eq!(bgra.len(), 64 * 64 * 4, "Buffer size must match 64x64x4 bytes (16 KB)");

        // 1. Góc trên bên trái (0, 0) phải hoàn toàn trong suốt (Alpha = 0)
        let top_left_alpha = bgra[3];
        assert_eq!(top_left_alpha, 0, "Corner (0, 0) must be 100% transparent alpha");

        // 2. Điểm chính giữa tâm (32, 32) phải đặc (Alpha > 200)
        let center_idx = (32 * 64 + 32) * 4;
        let center_alpha = bgra[center_idx + 3];
        assert!(center_alpha > 200, "Center (32, 32) must be solid alpha, got {}", center_alpha);
    }

    #[test]
    fn test_render_bubble_unread_badge_single_digit() {
        let opts = BubbleRenderOptions {
            size: 64,
            account_index: 2,
            is_hovered: false,
            has_pulse: false,
            pulse_progress: 0.0,
            unread_count: 5,
        };

        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for single digit");
        assert_eq!(bgra.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_render_bubble_unread_badge_double_digit() {
        let opts = BubbleRenderOptions {
            size: 64,
            account_index: 3,
            is_hovered: false,
            has_pulse: false,
            pulse_progress: 0.0,
            unread_count: 24,
        };

        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for double digit");
        assert_eq!(bgra.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_render_bubble_unread_badge_overflow() {
        let opts = BubbleRenderOptions {
            size: 64,
            account_index: 4,
            is_hovered: false,
            has_pulse: false,
            pulse_progress: 0.0,
            unread_count: 120,
        };

        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for overflow 99+");
        assert_eq!(bgra.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_render_bubble_pulse_wave() {
        let opts = BubbleRenderOptions {
            size: 64,
            account_index: 1,
            is_hovered: false,
            has_pulse: true,
            pulse_progress: 0.5,
            unread_count: 0,
        };

        let bgra = render_bubble_bgra(&opts).expect("render_bubble_bgra failed for pulse wave");
        assert_eq!(bgra.len(), 64 * 64 * 4);
    }
}
