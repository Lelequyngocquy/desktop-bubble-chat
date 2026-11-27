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

fn make_white_paint(alpha: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, alpha);
    paint.anti_alias = true;
    paint
}

/// Renders centered anti-aliased vector icon for a Liquid Glass dock droplet
#[allow(clippy::too_many_arguments)]
pub fn draw_dock_action_icon(
    pixmap: &mut Pixmap,
    action: DockActionId,
    is_active: bool,
    cx: f32,
    cy: f32,
    radius: f32,
    alpha_scale: f32,
    rotation_deg: f32,
    base_xf: Transform,
) {
    let icon_alpha = ((if is_active { 255.0 } else { 232.0 }) * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    if icon_alpha == 0 {
        return;
    }
    let s = (radius * 1.08) / 24.0;
    let stroke_w = (1.85 * s).clamp(1.15, 3.6);
    let xf = if rotation_deg.abs() > 0.01 {
        base_xf.pre_concat(Transform::from_rotate_at(rotation_deg, cx, cy))
    } else {
        base_xf
    };
    match action {
        DockActionId::Pin => draw_pin_icon(pixmap, cx, cy, s, stroke_w, is_active, icon_alpha, xf),
        DockActionId::AllowResize => draw_resize_icon(pixmap, cx, cy, s, stroke_w, is_active, icon_alpha, xf),
        DockActionId::PositionMode => draw_position_mode_icon(pixmap, cx, cy, s, stroke_w, is_active, icon_alpha, xf),
        DockActionId::Mute => draw_mute_icon(pixmap, cx, cy, s, stroke_w, is_active, icon_alpha, xf),
        DockActionId::Reload => draw_reload_icon(pixmap, cx, cy, s, stroke_w, icon_alpha, xf),
        DockActionId::HideBubble => draw_hide_bubble_icon(pixmap, cx, cy, s, stroke_w, icon_alpha, xf),
    }
}

/// Draws thumbtack icon: tilted outline when unpinned, upright filled when pinned
#[allow(clippy::too_many_arguments)]
fn draw_pin_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_pinned: bool, alpha: u8, base_xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let paint = make_white_paint(alpha);
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
            pixmap.fill_path(&head_path, &paint, FillRule::Winding, xf, None);
        } else {
            let stroke = make_stroke(stroke_w * 0.92);
            pixmap.stroke_path(&head_path, &paint, &stroke, xf, None);
        }
    }
    let mut needle_pb = PathBuilder::new();
    needle_pb.move_to(tx(12.0), ty(14.0));
    needle_pb.line_to(tx(12.0), ty(20.5));
    if let Some(needle_path) = needle_pb.finish() {
        let stroke = make_stroke(stroke_w);
        pixmap.stroke_path(&needle_path, &paint, &stroke, xf, None);
    }
}

/// Draws diagonal expand arrows indicating resizable / remember-size toggle
#[allow(clippy::too_many_arguments)]
fn draw_resize_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_active: bool, alpha: u8, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let paint = make_white_paint(alpha);
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
        pixmap.stroke_path(&path, &paint, &stroke, xf, None);
    }
}

/// Draws edge-dock icon for Auto-Tuck (OFF) and 4-way move arrows for Free mode (ON)
#[allow(clippy::too_many_arguments)]
fn draw_position_mode_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_free: bool, alpha: u8, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let paint = make_white_paint(alpha);
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
        pixmap.stroke_path(&path, &paint, &stroke, xf, None);
    }
}

/// Draws speaker with sound waves (Unmuted) or speaker with cross (Muted)
#[allow(clippy::too_many_arguments)]
fn draw_mute_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, is_muted: bool, alpha: u8, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let paint = make_white_paint(alpha);
    let mut cone_pb = PathBuilder::new();
    cone_pb.move_to(tx(4.5), ty(9.6));
    cone_pb.line_to(tx(8.0), ty(9.6));
    cone_pb.line_to(tx(12.2), ty(6.0));
    cone_pb.line_to(tx(12.2), ty(18.0));
    cone_pb.line_to(tx(8.0), ty(14.4));
    cone_pb.line_to(tx(4.5), ty(14.4));
    cone_pb.close();
    if let Some(cone_path) = cone_pb.finish() {
        pixmap.fill_path(&cone_path, &paint, FillRule::Winding, xf, None);
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
        pixmap.stroke_path(&detail_path, &paint, &stroke, xf, None);
    }
}

/// Draws circular refresh arc with directional arrowhead
fn draw_reload_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, alpha: u8, xf: Transform) {
    let paint = make_white_paint(alpha);
    let arc_r = 6.2 * s;
    let steps = 18usize;
    let start_deg = -40.0f32;
    let sweep_deg = 285.0f32;
    let mut pb = PathBuilder::new();
    for i in 0..=steps {
        let deg = start_deg + sweep_deg * (i as f32 / steps as f32);
        let rad = deg.to_radians();
        let px = cx + arc_r * rad.cos();
        let py = cy + arc_r * rad.sin();
        if i == 0 {
            pb.move_to(px, py);
        } else {
            pb.line_to(px, py);
        }
    }
    let tip_rad = start_deg.to_radians();
    let tip_x = cx + arc_r * tip_rad.cos();
    let tip_y = cy + arc_r * tip_rad.sin();
    pb.move_to(tip_x - 3.6 * s, tip_y + 0.3 * s);
    pb.line_to(tip_x, tip_y);
    pb.line_to(tip_x + 0.2 * s, tip_y - 3.6 * s);
    if let Some(path) = pb.finish() {
        let stroke = make_stroke(stroke_w);
        pixmap.stroke_path(&path, &paint, &stroke, xf, None);
    }
}

/// Draws minimalist eye-with-slash icon for hiding the active bubble
fn draw_hide_bubble_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, s: f32, stroke_w: f32, alpha: u8, xf: Transform) {
    let tx = |vx: f32| cx + (vx - 12.0) * s;
    let ty = |vy: f32| cy + (vy - 12.0) * s;
    let paint = make_white_paint(alpha);
    let mut eye_pb = PathBuilder::new();
    eye_pb.move_to(tx(4.2), ty(12.0));
    eye_pb.quad_to(tx(12.0), ty(5.6), tx(19.8), ty(12.0));
    eye_pb.quad_to(tx(12.0), ty(18.4), tx(4.2), ty(12.0));
    eye_pb.move_to(tx(5.5), ty(5.5));
    eye_pb.line_to(tx(18.5), ty(18.5));
    if let Some(eye_path) = eye_pb.finish() {
        let stroke = make_stroke(stroke_w * 0.95);
        pixmap.stroke_path(&eye_path, &paint, &stroke, xf, None);
    }
    let mut pupil_pb = PathBuilder::new();
    pupil_pb.push_circle(tx(12.0), ty(12.0), 1.8 * s);
    if let Some(pupil_path) = pupil_pb.finish() {
        pixmap.fill_path(&pupil_path, &paint, FillRule::Winding, xf, None);
    }
}
