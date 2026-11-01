use tiny_skia::*;

/// Vẽ hình con nhộng bo tròn (Pill / Capsule) cho Unread Badge nhiều chữ số
pub fn push_capsule(pb: &mut PathBuilder, left: f32, top: f32, w: f32, h: f32) {
    let right = left + w;
    let bottom = top + h;
    let r = h / 2.0;

    pb.move_to(left + r, top);
    pb.line_to(right - r, top);
    pb.quad_to(right, top, right, top + r);
    pb.quad_to(right, bottom, right - r, bottom);
    pb.line_to(left + r, bottom);
    pb.quad_to(left, bottom, left, top + r);
    pb.quad_to(left, top, left + r, top);
    pb.close();
}

/// Vẽ các ký tự số (0-9) và dấu '+' dưới dạng Vector Path siêu nhẹ, độ phân giải cao
pub fn draw_glyph(pb: &mut PathBuilder, ch: char, x: f32, y: f32, w: f32, h: f32) {
    let x0 = x;
    let x1 = x + w * 0.5;
    let x2 = x + w;
    let y0 = y;
    let y1 = y + h * 0.5;
    let y2 = y + h;

    match ch {
        '0' => {
            pb.move_to(x1, y0);
            pb.quad_to(x0, y0, x0, y1);
            pb.quad_to(x0, y2, x1, y2);
            pb.quad_to(x2, y2, x2, y1);
            pb.quad_to(x2, y0, x1, y0);
            pb.close();
        }
        '1' => {
            pb.move_to(x0 + w * 0.15, y0 + h * 0.25);
            pb.line_to(x1, y0);
            pb.line_to(x1, y2);
        }
        '2' => {
            pb.move_to(x0, y0 + h * 0.25);
            pb.quad_to(x0, y0, x1, y0);
            pb.quad_to(x2, y0, x2, y0 + h * 0.28);
            pb.quad_to(x2, y1, x0, y2);
            pb.line_to(x2, y2);
        }
        '3' => {
            pb.move_to(x0, y0 + h * 0.2);
            pb.quad_to(x1, y0, x2 - w * 0.1, y0 + h * 0.1);
            pb.quad_to(x2, y0 + h * 0.25, x1, y1);
            pb.quad_to(x2, y1 + h * 0.2, x2, y2 - h * 0.25);
            pb.quad_to(x1, y2, x0, y2 - h * 0.1);
        }
        '4' => {
            pb.move_to(x2 - w * 0.2, y2);
            pb.line_to(x2 - w * 0.2, y0);
            pb.line_to(x0, y1 + h * 0.1);
            pb.line_to(x2, y1 + h * 0.1);
        }
        '5' => {
            pb.move_to(x2, y0);
            pb.line_to(x0, y0);
            pb.line_to(x0, y1 - h * 0.05);
            pb.quad_to(x2, y1 - h * 0.05, x2, y2 - h * 0.25);
            pb.quad_to(x1, y2, x0, y2 - h * 0.1);
        }
        '6' => {
            pb.move_to(x2 - w * 0.1, y0 + h * 0.15);
            pb.quad_to(x0, y0 + h * 0.3, x0, y2 - h * 0.3);
            pb.quad_to(x0, y2, x1, y2);
            pb.quad_to(x2, y2, x2, y1 + h * 0.1);
            pb.quad_to(x2, y1 - h * 0.1, x1, y1 - h * 0.1);
            pb.quad_to(x0, y1 - h * 0.05, x0, y2 - h * 0.3);
        }
        '7' => {
            pb.move_to(x0, y0);
            pb.line_to(x2, y0);
            pb.line_to(x0 + w * 0.2, y2);
        }
        '8' => {
            pb.move_to(x1, y1);
            pb.quad_to(x0, y1 - h * 0.1, x0, y0 + h * 0.25);
            pb.quad_to(x0, y0, x1, y0);
            pb.quad_to(x2, y0, x2, y0 + h * 0.25);
            pb.quad_to(x2, y1 - h * 0.1, x1, y1);
            pb.quad_to(x0, y1 + h * 0.1, x0, y2 - h * 0.25);
            pb.quad_to(x0, y2, x1, y2);
            pb.quad_to(x2, y2, x2, y2 - h * 0.25);
            pb.quad_to(x2, y1 + h * 0.1, x1, y1);
        }
        '9' => {
            pb.move_to(x0 + w * 0.1, y2 - h * 0.15);
            pb.quad_to(x2, y2 - h * 0.3, x2, y0 + h * 0.3);
            pb.quad_to(x2, y0, x1, y0);
            pb.quad_to(x0, y0, x0, y1 - h * 0.1);
            pb.quad_to(x0, y1 + h * 0.1, x1, y1 + h * 0.1);
            pb.quad_to(x2, y1 + h * 0.05, x2, y0 + h * 0.3);
        }
        '+' => {
            pb.move_to(x1, y0 + h * 0.15);
            pb.line_to(x1, y2 - h * 0.15);
            pb.move_to(x0 + w * 0.1, y1);
            pb.line_to(x2 - w * 0.1, y1);
        }
        _ => {}
    }
}

/// Vẽ huy hiệu đếm tin nhắn chưa đọc (Unread Badge & Text)
pub fn draw_unread_badge(
    pixmap: &mut Pixmap,
    cx: f32,
    cy: f32,
    radius: f32,
    unread_count: u32,
) {
    if unread_count == 0 {
        return;
    }

    let badge_cx = cx + radius * 0.65;
    let badge_cy = cy - radius * 0.65;

    let mut badge_paint = Paint::default();
    badge_paint.set_color_rgba8(232, 17, 35, 255); // #E81123 Đỏ Windows Notification
    badge_paint.anti_alias = true;

    let mut badge_border_paint = Paint::default();
    badge_border_paint.set_color_rgba8(255, 255, 255, 255);
    badge_border_paint.anti_alias = true;

    let mut text_paint = Paint::default();
    text_paint.set_color_rgba8(255, 255, 255, 255);
    text_paint.anti_alias = true;

    let mut stroke_border = Stroke::default();
    stroke_border.width = 1.5;

    let mut stroke_text = Stroke::default();
    stroke_text.width = 1.25;
    stroke_text.line_cap = LineCap::Round;
    stroke_text.line_join = LineJoin::Round;

    if unread_count <= 9 {
        // Huy hiệu hình tròn cho 1 chữ số (1-9)
        let badge_r = 7.5;
        let mut pb = PathBuilder::new();
        pb.push_circle(badge_cx, badge_cy, badge_r);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &badge_paint, FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&path, &badge_border_paint, &stroke_border, Transform::identity(), None);
        }

        // Vẽ chữ số ở tâm
        let ch = char::from_digit(unread_count, 10).unwrap_or('1');
        let gw = 4.0;
        let gh = 7.0;
        let mut pb_text = PathBuilder::new();
        draw_glyph(&mut pb_text, ch, badge_cx - gw / 2.0, badge_cy - gh / 2.0, gw, gh);
        if let Some(text_path) = pb_text.finish() {
            pixmap.stroke_path(&text_path, &text_paint, &stroke_text, Transform::identity(), None);
        }
    } else if unread_count <= 99 {
        // Huy hiệu hình con nhộng bo tròn (Pill / Capsule) cho 2 chữ số (10-99)
        let pill_w = 19.0;
        let pill_h = 15.0;
        let left = badge_cx - pill_w / 2.0;
        let top = badge_cy - pill_h / 2.0;

        let mut pb = PathBuilder::new();
        push_capsule(&mut pb, left, top, pill_w, pill_h);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &badge_paint, FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&path, &badge_border_paint, &stroke_border, Transform::identity(), None);
        }

        // Vẽ 2 chữ số
        let d1 = char::from_digit(unread_count / 10, 10).unwrap_or('1');
        let d2 = char::from_digit(unread_count % 10, 10).unwrap_or('0');
        let gw = 3.6;
        let gh = 6.5;
        let gap = 1.0;
        let start_x = badge_cx - (gw * 2.0 + gap) / 2.0;
        let start_y = badge_cy - gh / 2.0;

        let mut pb_text = PathBuilder::new();
        draw_glyph(&mut pb_text, d1, start_x, start_y, gw, gh);
        draw_glyph(&mut pb_text, d2, start_x + gw + gap, start_y, gw, gh);
        if let Some(text_path) = pb_text.finish() {
            pixmap.stroke_path(&text_path, &text_paint, &stroke_text, Transform::identity(), None);
        }
    } else {
        // Huy hiệu hình con nhộng rộng hơn cho số lượng lớn ("99+")
        let pill_w = 23.0;
        let pill_h = 15.0;
        let left = badge_cx - pill_w / 2.0;
        let top = badge_cy - pill_h / 2.0;

        let mut pb = PathBuilder::new();
        push_capsule(&mut pb, left, top, pill_w, pill_h);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &badge_paint, FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&path, &badge_border_paint, &stroke_border, Transform::identity(), None);
        }

        // Vẽ 3 ký tự '9', '9', '+'
        let chars = ['9', '9', '+'];
        let gw = 3.2;
        let gh = 5.8;
        let gap = 0.8;
        let start_x = badge_cx - (gw * 3.0 + gap * 2.0) / 2.0;
        let start_y = badge_cy - gh / 2.0;

        let mut pb_text = PathBuilder::new();
        for (i, &ch) in chars.iter().enumerate() {
            let gx = start_x + (gw + gap) * i as f32;
            draw_glyph(&mut pb_text, ch, gx, start_y, gw, gh);
        }
        if let Some(text_path) = pb_text.finish() {
            stroke_text.width = 1.15;
            pixmap.stroke_path(&text_path, &text_paint, &stroke_text, Transform::identity(), None);
        }
    }
}
