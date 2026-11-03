use tiny_skia::*;

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

/// Vẽ huy hiệu đếm tin nhắn chưa đọc có hình tròn nhọn (pointed circle / teardrop pin) trỏ về phía Bong bóng:
/// - Khi Bubble ở mép trái màn hình (is_left_edge = true): Badge nằm ở phía BÊN PHẢI Bong bóng để không bị che khi tuck, mũi nhọn trỏ về bên trái (tâm bubble).
/// - Khi Bubble ở mép phải màn hình (is_left_edge = false): Badge nằm ở phía BÊN TRÁI Bong bóng để không bị che khi tuck, mũi nhọn trỏ về bên phải (tâm bubble).
pub fn draw_unread_badge(
    pixmap: &mut Pixmap,
    cx: f32,
    cy: f32,
    radius: f32,
    unread_count: u32,
    is_left_edge: bool,
) {
    if unread_count == 0 {
        return;
    }

    let mut badge_paint = Paint::default();
    badge_paint.set_color_rgba8(232, 17, 35, 255); // #E81123 Đỏ Windows Notification
    badge_paint.anti_alias = true;

    let mut badge_border_paint = Paint::default();
    badge_border_paint.set_color_rgba8(255, 255, 255, 255);
    badge_border_paint.anti_alias = true;

    let mut text_paint = Paint::default();
    text_paint.set_color_rgba8(255, 255, 255, 255);
    text_paint.anti_alias = true;

    // 3 phân cấp kích thước huy hiệu thích ứng linh hoạt theo 3 cỡ Bong bóng (sm: 48px, md: 56px, lg: 64px+)
    let is_small = radius <= 21.5;
    let is_large = radius > 25.5;

    let border_width = if is_small { 1.3 } else if is_large { 1.6 } else { 1.5 };
    let text_stroke_width = if is_small { 1.2 } else if is_large { 1.4 } else { 1.3 };

    let mut stroke_border = Stroke::default();
    stroke_border.width = border_width;
    stroke_border.line_join = LineJoin::Round;

    let mut stroke_text = Stroke::default();
    stroke_text.width = text_stroke_width;
    stroke_text.line_cap = LineCap::Round;
    stroke_text.line_join = LineJoin::Round;

    let gap_from_bubble = 3.0f32;

    if unread_count <= 9 {
        // ------------------------------------------------------------------------
        // 1. Huy hiệu hình tròn nhọn (Pointed Circle Pin) cho 1 chữ số (1-9)
        // ------------------------------------------------------------------------
        let (badge_r, tail_len, gw, gh) = if is_small {
            (6.8f32, 5.0f32, 3.8f32, 6.4f32)
        } else if is_large {
            (8.8f32, 5.8f32, 4.8f32, 8.0f32)
        } else {
            // Medium (md: 56px mặc định)
            (7.8f32, 5.4f32, 4.4f32, 7.2f32)
        };

        // Đặt badge hoàn toàn bên ngoài thân Bong bóng (không đè lên thân bubble)
        let dist_x = radius + gap_from_bubble + badge_r;
        let bcx = if is_left_edge { cx + dist_x } else { cx - dist_x };
        let bcy = cy - radius * 0.35;

        // Vector và góc hướng từ tâm badge về tâm Bong bóng (cx, cy)
        let dir_x = cx - bcx;
        let dir_y = cy - bcy;
        let angle = dir_y.atan2(dir_x);

        // Đỉnh mũi nhọn trỏ ra ngoài vòng tròn hướng về tâm Bong bóng
        let tip_x = bcx + (badge_r + tail_len) * angle.cos();
        let tip_y = bcy + (badge_r + tail_len) * angle.sin();

        // Hai điểm chân của mũi nhọn nằm trên viền hình tròn
        let delta = 36.0f32.to_radians();
        let p1_x = bcx + badge_r * (angle - delta).cos();
        let p1_y = bcy + badge_r * (angle - delta).sin();
        let p2_x = bcx + badge_r * (angle + delta).cos();
        let p2_y = bcy + badge_r * (angle + delta).sin();

        // Xây dựng đường bao khép kín liền mạch duy nhất (Single Continuous Polygon)
        let mut pb = PathBuilder::new();
        pb.move_to(p2_x, p2_y);

        // Vẽ cung tròn từ p2 quanh thân hình tròn tới p1 (chiếm ~288 độ)
        let span = 2.0 * std::f32::consts::PI - 2.0 * delta;
        let steps = 16;
        for i in 1..steps {
            let a = (angle + delta) + span * (i as f32 / steps as f32);
            pb.line_to(bcx + badge_r * a.cos(), bcy + badge_r * a.sin());
        }
        pb.line_to(p1_x, p1_y);

        // Kéo ra mũi nhọn trỏ vào bubble rồi khép về p2
        pb.line_to(tip_x, tip_y);
        pb.line_to(p2_x, p2_y);
        pb.close();

        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &badge_paint, FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&path, &badge_border_paint, &stroke_border, Transform::identity(), None);
        }

        // Vẽ chữ số chính xác ở tâm hình tròn
        let ch = char::from_digit(unread_count, 10).unwrap_or('1');
        let mut pb_text = PathBuilder::new();
        draw_glyph(&mut pb_text, ch, bcx - gw / 2.0, bcy - gh / 2.0, gw, gh);
        if let Some(text_path) = pb_text.finish() {
            pixmap.stroke_path(&text_path, &text_paint, &stroke_text, Transform::identity(), None);
        }
    } else if unread_count <= 99 {
        // ------------------------------------------------------------------------
        // 2. Huy hiệu hình con nhộng nhọn (Pointed Capsule) cho 2 chữ số (10-99)
        // ------------------------------------------------------------------------
        let (pill_w, pill_h, tail_dx, tail_dy, gw, gh, gap) = if is_small {
            (17.0f32, 13.0f32, 2.8f32, 3.2f32, 3.4f32, 5.8f32, 0.8f32)
        } else if is_large {
            (22.0f32, 17.0f32, 3.6f32, 4.0f32, 4.2f32, 7.8f32, 1.2f32)
        } else {
            // Medium (md: 56px mặc định)
            (19.5f32, 15.0f32, 3.2f32, 3.6f32, 3.8f32, 6.8f32, 1.0f32)
        };
        let r = pill_h / 2.0;

        // Đặt capsule hoàn toàn bên ngoài thân Bong bóng
        let dist_x = radius + gap_from_bubble + pill_w / 2.0;
        let bcx = if is_left_edge { cx + dist_x } else { cx - dist_x };
        let bcy = cy - radius * 0.35;

        let left = bcx - pill_w / 2.0;
        let right = bcx + pill_w / 2.0;
        let top = bcy - pill_h / 2.0;
        let bottom = bcy + pill_h / 2.0;

        let mut pb = PathBuilder::new();
        if is_left_edge {
            // Mũi nhọn ở góc dưới bên trái trỏ qua khe hở về phía bubble
            let tip_x = left - tail_dx;
            let tip_y = bottom + tail_dy;

            pb.move_to(left + r, top);
            pb.line_to(right - r, top);
            pb.quad_to(right, top, right, top + r);
            pb.quad_to(right, bottom, right - r, bottom);
            pb.line_to(left + r + 2.0, bottom);
            pb.line_to(tip_x, tip_y);
            pb.line_to(left, top + r + 1.2);
            pb.quad_to(left, top, left + r, top);
            pb.close();
        } else {
            // Mũi nhọn ở góc dưới bên phải trỏ qua khe hở về phía bubble
            let tip_x = right + tail_dx;
            let tip_y = bottom + tail_dy;

            pb.move_to(left + r, top);
            pb.line_to(right - r, top);
            pb.quad_to(right, top, right, top + r + 1.2);
            pb.line_to(tip_x, tip_y);
            pb.line_to(right - r - 2.0, bottom);
            pb.line_to(left + r, bottom);
            pb.quad_to(left, bottom, left, top + r);
            pb.quad_to(left, top, left + r, top);
            pb.close();
        }

        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &badge_paint, FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&path, &badge_border_paint, &stroke_border, Transform::identity(), None);
        }

        // Vẽ 2 chữ số
        let d1 = char::from_digit(unread_count / 10, 10).unwrap_or('1');
        let d2 = char::from_digit(unread_count % 10, 10).unwrap_or('0');
        let start_x = bcx - (gw * 2.0 + gap) / 2.0;
        let start_y = bcy - gh / 2.0;

        let mut pb_text = PathBuilder::new();
        draw_glyph(&mut pb_text, d1, start_x, start_y, gw, gh);
        draw_glyph(&mut pb_text, d2, start_x + gw + gap, start_y, gw, gh);
        if let Some(text_path) = pb_text.finish() {
            pixmap.stroke_path(&text_path, &text_paint, &stroke_text, Transform::identity(), None);
        }
    } else {
        // ------------------------------------------------------------------------
        // 3. Huy hiệu hình con nhộng nhọn cho số lượng lớn ("99+")
        // ------------------------------------------------------------------------
        let (pill_w, pill_h, tail_dx, tail_dy, gw, gh, gap) = if is_small {
            (21.0f32, 13.0f32, 2.8f32, 3.2f32, 3.0f32, 5.2f32, 0.6f32)
        } else if is_large {
            (27.0f32, 17.0f32, 3.6f32, 4.0f32, 3.8f32, 7.0f32, 1.0f32)
        } else {
            // Medium (md: 56px mặc định)
            (24.0f32, 15.0f32, 3.2f32, 3.6f32, 3.4f32, 6.0f32, 0.8f32)
        };
        let r = pill_h / 2.0;

        // Đặt capsule hoàn toàn bên ngoài thân Bong bóng
        let dist_x = radius + gap_from_bubble + pill_w / 2.0;
        let bcx = if is_left_edge { cx + dist_x } else { cx - dist_x };
        let bcy = cy - radius * 0.35;

        let left = bcx - pill_w / 2.0;
        let right = bcx + pill_w / 2.0;
        let top = bcy - pill_h / 2.0;
        let bottom = bcy + pill_h / 2.0;

        let mut pb = PathBuilder::new();
        if is_left_edge {
            let tip_x = left - tail_dx;
            let tip_y = bottom + tail_dy;

            pb.move_to(left + r, top);
            pb.line_to(right - r, top);
            pb.quad_to(right, top, right, top + r);
            pb.quad_to(right, bottom, right - r, bottom);
            pb.line_to(left + r + 2.0, bottom);
            pb.line_to(tip_x, tip_y);
            pb.line_to(left, top + r + 1.2);
            pb.quad_to(left, top, left + r, top);
            pb.close();
        } else {
            let tip_x = right + tail_dx;
            let tip_y = bottom + tail_dy;

            pb.move_to(left + r, top);
            pb.line_to(right - r, top);
            pb.quad_to(right, top, right, top + r + 1.2);
            pb.line_to(tip_x, tip_y);
            pb.line_to(right - r - 2.0, bottom);
            pb.line_to(left + r, bottom);
            pb.quad_to(left, bottom, left, top + r);
            pb.quad_to(left, top, left + r, top);
            pb.close();
        }

        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &badge_paint, FillRule::Winding, Transform::identity(), None);
            pixmap.stroke_path(&path, &badge_border_paint, &stroke_border, Transform::identity(), None);
        }

        // Vẽ 3 ký tự '9', '9', '+'
        let chars = ['9', '9', '+'];
        let start_x = bcx - (gw * 3.0 + gap * 2.0) / 2.0;
        let start_y = bcy - gh / 2.0;

        let mut pb_text = PathBuilder::new();
        for (i, &ch) in chars.iter().enumerate() {
            let gx = start_x + (gw + gap) * i as f32;
            draw_glyph(&mut pb_text, ch, gx, start_y, gw, gh);
        }
        if let Some(text_path) = pb_text.finish() {
            let text_stroke_w = if is_small { 1.15 } else if is_large { 1.35 } else { 1.25 };
            let mut st = stroke_text.clone();
            st.width = text_stroke_w;
            pixmap.stroke_path(&text_path, &text_paint, &st, Transform::identity(), None);
        }
    }
}
