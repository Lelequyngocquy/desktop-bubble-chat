use tiny_skia::*;

use super::model::DockActionId;

fn make_stroke(width: f32) -> Stroke {
    Stroke {
        width: width.max(1.0),
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    }
}

fn make_rgba_paint(r: u8, g: u8, b: u8, alpha: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, alpha);
    paint.anti_alias = true;
    paint
}

/// Renders centered anti-aliased vector icon with subtle dark shadow for translucent glass legibility
#[allow(clippy::too_many_arguments)]
pub fn draw_dock_action_icon(
    pixmap: &mut Pixmap,
    action: DockActionId,
    is_active: bool,
    cx: f32,
    cy: f32,
    radius: f32,
    alpha_scale: f32,
    base_xf: Transform,
) {
    let clamped_alpha = alpha_scale.clamp(0.0, 1.0);
    let icon_alpha = ((if is_active { 255.0 } else { 244.0 }) * clamped_alpha).round() as u8;
    if icon_alpha == 0 {
        return;
    }
    let shadow_alpha = ((if is_active { 78.0 } else { 142.0 }) * clamped_alpha).round() as u8;
    let s = (radius * 1.08) / 24.0;
    let stroke_w = (1.85 * s).clamp(1.15, 3.6);
    let shadow_dy = (0.75 * (radius / 15.0)).max(0.5);
    let shadow_xf = base_xf.post_translate(0.0, shadow_dy);
    let shadow_paint = make_rgba_paint(12, 16, 24, shadow_alpha);
    let white_paint = make_rgba_paint(255, 255, 255, icon_alpha);
    draw_action_glyph(pixmap, action, is_active, cx, cy, s, stroke_w * 1.08, &shadow_paint, shadow_xf);
    draw_action_glyph(pixmap, action, is_active, cx, cy, s, stroke_w, &white_paint, base_xf);
}

#[allow(clippy::too_many_arguments)]
fn draw_action_glyph(
    pixmap: &mut Pixmap,
    action: DockActionId,
    is_active: bool,
    cx: f32,
    cy: f32,
    s: f32,
    stroke_w: f32,
    paint: &Paint<'_>,
    xf: Transform,
) {
    match action {
        DockActionId::Pin => draw_pin_icon(pixmap, cx, cy, s, stroke_w, is_active, paint, xf),
        DockActionId::AllowResize => draw_resize_icon(pixmap, cx, cy, s, stroke_w, is_active, paint, xf),
        DockActionId::PositionMode => draw_position_mode_icon(pixmap, cx, cy, s, stroke_w, is_active, paint, xf),
        DockActionId::Mute => draw_mute_icon(pixmap, cx, cy, s, stroke_w, is_active, paint, xf),
        DockActionId::HideBubble => draw_hide_bubble_icon(pixmap, cx, cy, s, stroke_w, paint, xf),
    }
}

/// Draws thumbtack icon: tilted outline when unpinned, upright filled when pinned
#[allow(clippy::too_many_arguments)]
fn draw_pin_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_pinned: bool, paint: &Paint<'_>, base_xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let xf = if is_pinned { base_xf } else { base_xf.pre_concat(Transform::from_rotate_at(-35.0, cx, cy)) };
    let mut head_pb = PathBuilder::new();
    head_pb.move_to(tx(8.2), ty(4.2));
    head_pb.line_to(tx(15.8), ty(4.2));
    head_pb.line_to(tx(14.5), ty(6.2));
    head_pb.line_to(tx(14.5), ty(10.8));
    head_pb.line_to(tx(17.5), ty(14.0));
    head_pb.line_to(tx(6.5), ty(14.0));
    head_pb.line_to(tx(9.5), ty(10.8));
    head_pb.line_to(tx(9.5), ty(6.2));
    head_pb.close();
    if let Some(head_path) = head_pb.finish() {
        if is_pinned {
            pixmap.fill_path(&head_path, paint, FillRule::Winding, xf, None);
        } else {
            let stroke = make_stroke(stroke_w * 0.92);
            pixmap.stroke_path(&head_path, paint, &stroke, xf, None);
        }
    }
    let mut needle_pb = PathBuilder::new();
    needle_pb.move_to(tx(12.0), ty(14.0));
    needle_pb.line_to(tx(12.0), ty(20.5));
    if let Some(needle_path) = needle_pb.finish() {
        let stroke = make_stroke(stroke_w);
        pixmap.stroke_path(&needle_path, paint, &stroke, xf, None);
    }
}

/// Draws diagonal expand arrows indicating resizable / remember-size toggle
#[allow(clippy::too_many_arguments)]
fn draw_resize_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_active: bool, paint: &Paint<'_>, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let w = if is_active { stroke_w * 1.12 } else { stroke_w };
    let mut pb = PathBuilder::new();
    pb.move_to(tx(14.0), ty(5.5));
    pb.line_to(tx(18.5), ty(5.5));
    pb.line_to(tx(18.5), ty(10.0));
    pb.move_to(tx(18.5), ty(5.5));
    pb.line_to(tx(13.5), ty(10.5));
    pb.move_to(tx(10.0), ty(18.5));
    pb.line_to(tx(5.5), ty(18.5));
    pb.line_to(tx(5.5), ty(14.0));
    pb.move_to(tx(5.5), ty(18.5));
    pb.line_to(tx(10.5), ty(13.5));
    if let Some(path) = pb.finish() {
        let stroke = make_stroke(w);
        pixmap.stroke_path(&path, paint, &stroke, xf, None);
    }
}

/// Draws edge-dock icon for Auto-Tuck (OFF) and 4-way move arrows for Free mode (ON)
#[allow(clippy::too_many_arguments)]
fn draw_position_mode_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_free: bool, paint: &Paint<'_>, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let mut pb = PathBuilder::new();
    if is_free {
        pb.move_to(tx(12.0), ty(4.5));
        pb.line_to(tx(12.0), ty(19.5));
        pb.move_to(tx(4.5), ty(12.0));
        pb.line_to(tx(19.5), ty(12.0));
        pb.move_to(tx(9.5), ty(7.0));
        pb.line_to(tx(12.0), ty(4.5));
        pb.line_to(tx(14.5), ty(7.0));
        pb.move_to(tx(9.5), ty(17.0));
        pb.line_to(tx(12.0), ty(19.5));
        pb.line_to(tx(14.5), ty(17.0));
        pb.move_to(tx(7.0), ty(9.5));
        pb.line_to(tx(4.5), ty(12.0));
        pb.line_to(tx(7.0), ty(14.5));
        pb.move_to(tx(17.0), ty(9.5));
        pb.line_to(tx(19.5), ty(12.0));
        pb.line_to(tx(17.0), ty(14.5));
    } else {
        pb.move_to(tx(5.5), ty(5.5));
        pb.line_to(tx(5.5), ty(18.5));
        pb.move_to(tx(18.5), ty(12.0));
        pb.line_to(tx(9.5), ty(12.0));
        pb.move_to(tx(13.2), ty(8.3));
        pb.line_to(tx(9.5), ty(12.0));
        pb.line_to(tx(13.2), ty(15.7));
    }
    if let Some(path) = pb.finish() {
        let stroke = make_stroke(stroke_w);
        pixmap.stroke_path(&path, paint, &stroke, xf, None);
    }
}

/// Draws speaker with sound waves (Unmuted) or speaker with cross (Muted)
#[allow(clippy::too_many_arguments)]
fn draw_mute_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_muted: bool, paint: &Paint<'_>, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let mut cone_pb = PathBuilder::new();
    cone_pb.move_to(tx(4.5), ty(9.6));
    cone_pb.line_to(tx(8.0), ty(9.6));
    cone_pb.line_to(tx(12.2), ty(6.0));
    cone_pb.line_to(tx(12.2), ty(18.0));
    cone_pb.line_to(tx(8.0), ty(14.4));
    cone_pb.line_to(tx(4.5), ty(14.4));
    cone_pb.close();
    if let Some(cone_path) = cone_pb.finish() {
        pixmap.fill_path(&cone_path, paint, FillRule::Winding, xf, None);
    }
    let mut detail_pb = PathBuilder::new();
    if is_muted {
        detail_pb.move_to(tx(15.2), ty(9.2));
        detail_pb.line_to(tx(20.0), ty(14.8));
        detail_pb.move_to(tx(20.0), ty(9.2));
        detail_pb.line_to(tx(15.2), ty(14.8));
    } else {
        detail_pb.move_to(tx(15.0), ty(9.2));
        detail_pb.quad_to(tx(16.8), ty(12.0), tx(15.0), ty(14.8));
        detail_pb.move_to(tx(17.5), ty(7.0));
        detail_pb.quad_to(tx(20.5), ty(12.0), tx(17.5), ty(17.0));
    }
    if let Some(detail_path) = detail_pb.finish() {
        let stroke = make_stroke(stroke_w);
        pixmap.stroke_path(&detail_path, paint, &stroke, xf, None);
    }
}

/// Draws minimalist eye-with-slash icon for hiding the active bubble
fn draw_hide_bubble_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, paint: &Paint<'_>, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let mut eye_pb = PathBuilder::new();
    eye_pb.move_to(tx(4.2), ty(12.0));
    eye_pb.quad_to(tx(12.0), ty(5.6), tx(19.8), ty(12.0));
    eye_pb.quad_to(tx(12.0), ty(18.4), tx(4.2), ty(12.0));
    eye_pb.move_to(tx(5.5), ty(5.5));
    eye_pb.line_to(tx(18.5), ty(18.5));
    if let Some(eye_path) = eye_pb.finish() {
        let stroke = make_stroke(stroke_w * 0.95);
        pixmap.stroke_path(&eye_path, paint, &stroke, xf, None);
    }
    let mut pupil_pb = PathBuilder::new();
    pupil_pb.push_circle(tx(12.0), ty(12.0), 1.8 * s);
    if let Some(pupil_path) = pupil_pb.finish() {
        pixmap.fill_path(&pupil_path, paint, FillRule::Winding, xf, None);
    }
}
