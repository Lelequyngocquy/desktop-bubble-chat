use tiny_skia::*;

use crate::presentation::bubble::renderer::rgba_to_bgra;
use super::animator::DropletFrameState;
use super::icons::draw_dock_action_icon;
use super::layout::ActionDockLayout;
use super::model::{DockActionId, DockActionStateSnapshot, DockButtonTone};

/// Input payload for rendering the entire Liquid Glass ActionDock frame into BGRA
#[derive(Debug, Clone)]
pub struct DockRenderOptions {
    pub layout: ActionDockLayout,
    pub state: DockActionStateSnapshot,
    pub frames: [DropletFrameState; 6],
    pub hovered_action: Option<DockActionId>,
}

fn scale_alpha(base: u8, factor: f32) -> u8 {
    ((base as f32) * factor.clamp(0.0, 1.0)).round().clamp(0.0, 255.0) as u8
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let clamped = t.clamp(0.0, 1.0);
    ((a as f32) + ((b as f32) - (a as f32)) * clamped).round().clamp(0.0, 255.0) as u8
}

type Rgba = (u8, u8, u8, u8);

/// Resolves top-left and bottom-right Liquid Glass sphere gradient endpoints
fn glass_body_colors(tone: DockButtonTone, hover_t: f32) -> (Rgba, Rgba) {
    match tone {
        DockButtonTone::Neutral => (
            (lerp_u8(78, 102, hover_t), lerp_u8(84, 110, hover_t), lerp_u8(96, 124, hover_t), lerp_u8(202, 222, hover_t)),
            (lerp_u8(44, 58, hover_t), lerp_u8(48, 64, hover_t), lerp_u8(56, 74, hover_t), lerp_u8(216, 232, hover_t)),
        ),
        DockButtonTone::ActiveAccent => (
            (lerp_u8(56, 86, hover_t), lerp_u8(152, 174, hover_t), 255, lerp_u8(226, 240, hover_t)),
            (lerp_u8(10, 24, hover_t), lerp_u8(114, 134, hover_t), lerp_u8(235, 250, hover_t), lerp_u8(238, 248, hover_t)),
        ),
        DockButtonTone::MutedWarning => (
            (lerp_u8(245, 252, hover_t), lerp_u8(140, 158, hover_t), lerp_u8(24, 46, hover_t), lerp_u8(226, 240, hover_t)),
            (lerp_u8(217, 232, hover_t), lerp_u8(78, 94, hover_t), lerp_u8(36, 52, hover_t), lerp_u8(238, 248, hover_t)),
        ),
        DockButtonTone::DestructiveHover => (
            (248, 113, 113, 226),
            (220, 38, 38, 240),
        ),
    }
}

/// Builds an elliptical specular highlight path seated in the upper third of the droplet
fn build_specular_sheen_path(cx: f32, cy: f32, rx: f32, ry: f32) -> Option<Path> {
    let k = 0.5522848f32;
    let kx = rx * k;
    let ky = ry * k;
    let mut pb = PathBuilder::new();
    pb.move_to(cx, cy - ry);
    pb.cubic_to(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy);
    pb.cubic_to(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry);
    pb.cubic_to(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy);
    pb.cubic_to(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry);
    pb.close();
    pb.finish()
}

/// Renders all 6 Liquid Glass circular droplets into a premultiplied 32-bit BGRA buffer
pub fn render_action_dock_bgra(opts: &DockRenderOptions) -> Option<Vec<u8>> {
    let w = opts.layout.canvas_width;
    let h = opts.layout.canvas_height;
    let mut pixmap = Pixmap::new(w, h)?;
    pixmap.fill(Color::TRANSPARENT);
    let scale = opts.layout.scale_factor as f32;
    for slot in &opts.layout.slots {
        let frame = opts.frames[slot.action.index()];
        if frame.alpha <= 0.005 {
            continue;
        }
        let cx = slot.local_cx;
        let cy = slot.local_cy;
        let r = slot.radius;
        let dy = frame.offset_y_norm * 10.0 * scale;
        let group_xf = Transform::from_translate(cx, cy + dy)
            .pre_scale(frame.scale, frame.scale)
            .pre_translate(-cx, -cy);
        let is_hovered = opts.hovered_action == Some(slot.action);
        let is_active = opts.state.is_action_active(slot.action);
        let tone = opts.state.button_tone(slot.action, is_hovered);
        // Layer 1: Soft ambient drop shadow
        let mut shadow_pb = PathBuilder::new();
        shadow_pb.push_circle(cx, cy + 1.8 * scale, r + 0.6 * scale);
        if let Some(shadow_path) = shadow_pb.finish() {
            let mut shadow_paint = Paint::default();
            shadow_paint.set_color_rgba8(0, 0, 0, scale_alpha(52, frame.alpha));
            shadow_paint.anti_alias = true;
            pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, group_xf, None);
        }
        // Layer 2: Frosted translucent glass body gradient
        let (top_c, bot_c) = glass_body_colors(tone, frame.hover_t);
        let mut body_pb = PathBuilder::new();
        body_pb.push_circle(cx, cy, r);
        if let Some(body_path) = body_pb.finish() {
            if let Some(body_shader) = LinearGradient::new(
                Point::from_xy(cx - r * 0.55, cy - r),
                Point::from_xy(cx + r * 0.55, cy + r),
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(top_c.0, top_c.1, top_c.2, scale_alpha(top_c.3, frame.alpha))),
                    GradientStop::new(1.0, Color::from_rgba8(bot_c.0, bot_c.1, bot_c.2, scale_alpha(bot_c.3, frame.alpha))),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let body_paint = Paint { shader: body_shader, anti_alias: true, ..Default::default() };
                pixmap.fill_path(&body_path, &body_paint, FillRule::Winding, group_xf, None);
            }
            // Layer 3: Upper-left inner specular liquid sheen
            let sheen_cy = cy - r * 0.36;
            let sheen_rx = r * 0.64;
            let sheen_ry = r * 0.34;
            if let Some(sheen_path) = build_specular_sheen_path(cx - r * 0.08, sheen_cy, sheen_rx, sheen_ry)
                && let Some(sheen_shader) = LinearGradient::new(
                    Point::from_xy(cx, sheen_cy - sheen_ry),
                    Point::from_xy(cx, sheen_cy + sheen_ry),
                    vec![
                        GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, scale_alpha(lerp_u8(84, 138, frame.specular_boost), frame.alpha))),
                        GradientStop::new(1.0, Color::from_rgba8(255, 255, 255, 0)),
                    ],
                    SpreadMode::Pad,
                    Transform::identity(),
                ) {
                    let sheen_paint = Paint { shader: sheen_shader, anti_alias: true, ..Default::default() };
                    pixmap.fill_path(&sheen_path, &sheen_paint, FillRule::Winding, group_xf, None);
                }
            // Layer 4: Outer dark contrast hair-rim + inner diagonal specular glass rim
            let mut outer_rim_pb = PathBuilder::new();
            outer_rim_pb.push_circle(cx, cy, r + 0.3 * scale);
            if let Some(outer_path) = outer_rim_pb.finish() {
                let mut dark_rim = Paint::default();
                dark_rim.set_color_rgba8(0, 0, 0, scale_alpha(44, frame.alpha));
                dark_rim.anti_alias = true;
                let dark_stroke = Stroke { width: (0.7 * scale).max(0.6), ..Default::default() };
                pixmap.stroke_path(&outer_path, &dark_rim, &dark_stroke, group_xf, None);
            }
            let top_rim_a = if is_active || is_hovered { 196 } else { 140 };
            if let Some(rim_shader) = LinearGradient::new(
                Point::from_xy(cx - r, cy - r),
                Point::from_xy(cx + r, cy + r),
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, scale_alpha(top_rim_a, frame.alpha))),
                    GradientStop::new(0.55, Color::from_rgba8(255, 255, 255, scale_alpha(48, frame.alpha))),
                    GradientStop::new(1.0, Color::from_rgba8(255, 255, 255, scale_alpha(26, frame.alpha))),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let rim_paint = Paint { shader: rim_shader, anti_alias: true, ..Default::default() };
                let rim_stroke = Stroke { width: (1.15 * scale).max(1.0), ..Default::default() };
                pixmap.stroke_path(&body_path, &rim_paint, &rim_stroke, group_xf, None);
            }
        }
        // Layer 5: Centered crisp vector icon
        draw_dock_action_icon(
            &mut pixmap,
            slot.action,
            is_active,
            cx,
            cy,
            r,
            frame.alpha,
            frame.icon_rotation_deg,
            group_xf,
        );
    }
    Some(rgba_to_bgra(pixmap.data()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::ScreenMetrics;
    use super::super::animator::DOCK_HOVER_SCALE;
    use super::super::layout::compute_action_dock_layout;

    #[test]
    fn test_render_action_dock_zero_clipping_across_sizes_and_dpi() {
        for (logical_bubble, scale) in [(48u32, 1.0f64), (56, 1.0), (64, 1.0), (56, 1.5), (64, 2.0)] {
            let bubble_px = (logical_bubble as f64 * scale).round() as u32;
            let screen = ScreenMetrics {
                x: 0,
                y: 0,
                width: (1920.0 * scale) as i32,
                height: (1080.0 * scale) as i32,
                scale_factor: scale,
            };
            let layout = compute_action_dock_layout(20, 120, bubble_px, &screen, 12.0, 60.0);
            let mut frames = [DropletFrameState::default(); 6];
            frames[0].scale = DOCK_HOVER_SCALE;
            frames[0].hover_t = 1.0;
            frames[0].specular_boost = 0.5;
            frames[5].scale = DOCK_HOVER_SCALE;
            frames[5].hover_t = 1.0;
            let opts = DockRenderOptions {
                layout: layout.clone(),
                state: DockActionStateSnapshot {
                    account_index: 1,
                    is_pinned: true,
                    is_resizable: true,
                    is_free_position: false,
                    is_muted: true,
                    bubble_size_px: bubble_px,
                    scale_factor: scale,
                },
                frames,
                hovered_action: Some(DockActionId::HideBubble),
            };
            let bgra = render_action_dock_bgra(&opts).expect("render_action_dock_bgra failed");
            let w = layout.canvas_width as usize;
            let h = layout.canvas_height as usize;
            assert_eq!(bgra.len(), w * h * 4);
            // Verify all 4 outer border edges stay 100% transparent (no shadow or hover clipping)
            for x in 0..w {
                assert_eq!(bgra[x * 4 + 3], 0, "top border clipped at x={} scale={}", x, scale);
                assert_eq!(bgra[((h - 1) * w + x) * 4 + 3], 0, "bottom border clipped at x={} scale={}", x, scale);
            }
            for y in 0..h {
                assert_eq!(bgra[(y * w) * 4 + 3], 0, "left border clipped at y={} scale={}", y, scale);
                assert_eq!(bgra[(y * w + (w - 1)) * 4 + 3], 0, "right border clipped at y={} scale={}", y, scale);
            }
            // Verify each droplet center renders opaque glass
            for slot in &layout.slots {
                let cx = slot.local_cx.round() as usize;
                let cy = slot.local_cy.round() as usize;
                let center_alpha = bgra[(cy * w + cx) * 4 + 3];
                assert!(center_alpha > 180, "droplet {:?} center too faint: {}", slot.action, center_alpha);
            }
        }
    }

    #[test]
    fn test_render_action_dock_distinguishes_active_and_neutral_states() {
        let screen = ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let layout = compute_action_dock_layout(20, 120, 56, &screen, 12.0, 60.0);
        let off = render_action_dock_bgra(&DockRenderOptions {
            layout: layout.clone(),
            state: DockActionStateSnapshot::default(),
            frames: [DropletFrameState::default(); 6],
            hovered_action: None,
        }).unwrap();
        let on = render_action_dock_bgra(&DockRenderOptions {
            layout,
            state: DockActionStateSnapshot { is_pinned: true, is_resizable: true, is_free_position: true, is_muted: true, ..Default::default() },
            frames: [DropletFrameState::default(); 6],
            hovered_action: Some(DockActionId::HideBubble),
        }).unwrap();
        assert_ne!(off, on);
    }
}
