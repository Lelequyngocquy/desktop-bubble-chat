use tiny_skia::*;

/// Xây dựng và vẽ biểu tượng vector chuẩn Messenger (theo MessengerIcon.tsx)
/// Bao gồm vỏ bóng thoại thoại có đuôi nhọn và vết cắt tia sét ở giữa với FillRule::EvenOdd
pub fn draw_messenger_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, radius: f32) {
    let icon_size = radius * 1.05; // Tỉ lệ chuẩn 50% đường kính bubble
    let s = icon_size / 36.0;

    let tx = |vx: f32| cx + (vx - 18.0) * s;
    let ty = |vy: f32| cy + (vy - 18.0) * s;

    let mut pb = PathBuilder::new();

    // Subpath 1: Vỏ bóng thoại Messenger (có đuôi nhọn chỉ về góc dưới bên trái)
    pb.move_to(tx(18.0), ty(1.0));
    pb.cubic_to(tx(8.61), ty(1.0), tx(1.0), ty(8.16), tx(1.0), ty(17.0));
    pb.cubic_to(tx(1.0), ty(22.04), tx(3.50), ty(26.54), tx(7.43), ty(29.45));
    pb.line_to(tx(7.43), ty(35.0));
    pb.line_to(tx(12.72), ty(32.09));
    pb.cubic_to(tx(14.40), ty(32.56), tx(16.18), ty(32.81), tx(18.0), ty(32.81));
    pb.cubic_to(tx(27.39), ty(32.81), tx(35.0), ty(25.65), tx(35.0), ty(16.81));
    pb.cubic_to(tx(35.0), ty(8.16), tx(27.39), ty(1.0), tx(18.0), ty(1.0));
    pb.close();

    // Subpath 2: Tia sét cutout bên trong
    pb.move_to(tx(19.75), ty(22.62));
    pb.line_to(tx(15.41), ty(17.99));
    pb.line_to(tx(6.93), ty(22.62));
    pb.line_to(tx(16.26), ty(12.71));
    pb.line_to(tx(20.71), ty(17.34));
    pb.line_to(tx(29.08), ty(12.71));
    pb.line_to(tx(19.75), ty(22.62));
    pb.close();

    if let Some(icon_path) = pb.finish() {
        let mut icon_paint = Paint::default();
        icon_paint.set_color_rgba8(255, 255, 255, 255);
        icon_paint.anti_alias = true;

        pixmap.fill_path(
            &icon_path,
            &icon_paint,
            FillRule::EvenOdd,
            Transform::identity(),
            None,
        );
    }
}
