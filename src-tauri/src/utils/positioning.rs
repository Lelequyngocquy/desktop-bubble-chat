use tauri::{AppHandle, Emitter, Manager, Monitor, PhysicalPosition};


#[derive(Debug, Clone, Copy)]
pub struct ScreenMetrics {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f64,
}

impl ScreenMetrics {
    pub fn from_monitor(monitor: &Monitor) -> Self {
        let pos = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor();
        Self {
            x: pos.x,
            y: pos.y,
            width: size.width as i32,
            height: size.height as i32,
            scale_factor: scale,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct BubbleConfig {
    pub size: f64,           
    pub tuck_ratio: f64,     
    pub appear_ratio: f64,     
    pub margin: f64,         
    pub taskbar_margin: f64, 
    pub dismiss_margin: f64, 
    pub chat_width: f64,     
    pub chat_height: f64,    
    pub chat_gap: f64,       
}

pub const DEFAULT_BUBBLE_CONFIG: BubbleConfig = BubbleConfig {
    size: 64.0, 
    tuck_ratio: 0.8, 
    appear_ratio: 0.0, 
    margin: 12.0, 
    taskbar_margin: 60.0, 
    dismiss_margin: 80.0, 
    chat_width: 460.0,
    chat_height: 620.0,
    chat_gap: 8.0,
};


#[derive(Debug, Clone, Copy)]
pub struct SnapResult {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
}


/// Tính toạ độ mép trái/phải dựa trên tỉ lệ lấn mép (appear_ratio vs tuck_ratio)
pub fn calculate_bubble_edge_x(
    screen: &ScreenMetrics,
    current_x: i32,
    ratio: f64,
    config: &BubbleConfig,
) -> i32 {
    let scale = screen.scale_factor;
    let bubble_width = (config.size * scale) as i32;
    let tuck = (bubble_width as f64 * ratio) as i32;
    let mid_x = screen.x + screen.width / 2;

    if current_x + bubble_width / 2 < mid_x {
        // Hít mép trái
        screen.x - tuck
    } else {
        // Hít mép phải
        screen.x + screen.width - bubble_width + tuck
    }
}

/// Tính toạ độ khi hover vào/ra khỏi bong bóng nổi (appear_ratio vs tuck_ratio)
pub fn calculate_bubble_hover_pos(
    screen: &ScreenMetrics,
    current_x: i32,
    current_y: i32,
    is_hovered: bool,
    config: &BubbleConfig,
) -> PhysicalPosition<i32> {
    let ratio = if is_hovered {
        config.appear_ratio
    } else {
        config.tuck_ratio
    };
    let target_x = calculate_bubble_edge_x(screen, current_x, ratio, config);

    let scale = screen.scale_factor;
    let bubble_width = (config.size * scale) as i32;
    let margin = (config.margin * scale) as i32;
    let taskbar_margin = (config.taskbar_margin * scale) as i32;
    let min_y = screen.y + margin;
    let max_y = (screen.y + screen.height - bubble_width - taskbar_margin).max(min_y);
    let target_y = current_y.clamp(min_y, max_y);

    PhysicalPosition {
        x: target_x,
        y: target_y,
    }
}

pub fn calculate_bubble_snap(
    screen: &ScreenMetrics,
    win_pos: PhysicalPosition<i32>,
    config: &BubbleConfig,
) -> SnapResult {
    let scale = screen.scale_factor;
    let bubble_width = (config.size * scale) as i32;
    let margin = (config.margin * scale) as i32;
    let taskbar_margin = (config.taskbar_margin * scale) as i32;

    // 1. Kiểm tra kéo sát đáy màn hình để ẩn (Dismiss to tray)
    let bottom_threshold = screen.y + screen.height - (config.dismiss_margin * scale) as i32;
    if win_pos.y >= bottom_threshold {
        return SnapResult {
            target_x: win_pos.x,
            target_y: win_pos.y,
            should_dismiss: true,
        };
    }

    // 2. Tính toạ độ hít viền trái hoặc phải (mặc định lấn ra ngoài theo tuck_ratio)
    let target_x = calculate_bubble_edge_x(screen, win_pos.x, config.tuck_ratio, config);

    // 3. Giới hạn trục Y không lọt ra ngoài mép trên và thanh Taskbar
    let min_y = screen.y + margin;
    let max_y = (screen.y + screen.height - bubble_width - taskbar_margin).max(min_y);
    let target_y = win_pos.y.clamp(min_y, max_y);

    SnapResult {
        target_x,
        target_y,
        should_dismiss: false,
    }
}

pub fn calculate_bubble_default_pos(
    screen: &ScreenMetrics,
    config: &BubbleConfig,
) -> PhysicalPosition<i32> {
    let scale = screen.scale_factor;
    let bubble_w = (config.size * scale) as i32;
    let margin = (config.margin * scale) as i32;
    let taskbar_margin = (config.taskbar_margin * scale) as i32;

    // Vị trí mặc định ở mép phải với tuck_ratio
    let target_x = calculate_bubble_edge_x(screen, screen.x + screen.width, config.tuck_ratio, config);
    let min_y = screen.y + margin;
    let max_y = (screen.y + screen.height - bubble_w - taskbar_margin).max(min_y);
    let target_y = (screen.y + screen.height / 3).clamp(min_y, max_y);

    PhysicalPosition {
        x: target_x,
        y: target_y,
    }
}

/// Trả về label định danh cửa sổ Bubble theo số thứ tự tài khoản
pub fn get_bubble_window_label(account_index: u32) -> String {
    format!("bubble-{:02}", account_index)
}

/// Tìm hoặc khởi tạo WebviewWindow nổi độc lập cho Bubble của tài khoản chỉ định
pub fn get_or_create_bubble_window(
    app: &AppHandle,
    account_index: u32,
) -> Result<tauri::WebviewWindow, String> {
    let label = get_bubble_window_label(account_index);

    // 1. Kiểm tra cửa sổ đã tồn tại (được khởi tạo tĩnh an toàn từ tauri.conf.json trên main thread)
    if let Some(win) = app.get_webview_window(&label) {
        return Ok(win);
    }

    // 2. Khởi tạo WebviewWindow Bubble dự phòng nếu chưa có
    let url_str = format!("index.html?window=bubble&account={}", account_index);
    let window = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App(url_str.into()),
    )
    .title(format!("DesktopBubbleChat - Bubble {:02}", account_index))
    .inner_size(64.0, 64.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(window)
}

/// Lấy danh sách tọa độ (vị trí, chiều rộng, chiều cao) của tất cả Bubble đang hiển thị
pub fn get_other_visible_bubble_rects(
    app: &AppHandle,
    exclude_label: Option<&str>,
) -> Vec<(PhysicalPosition<i32>, i32, i32)> {
    let mut rects = Vec::new();

    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        let win = app.get_webview_window(&label);

        if let Some(w) = win {
            if Some(w.label()) == exclude_label {
                continue;
            }
            if w.is_visible().unwrap_or(false) {
                if let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) {
                    rects.push((pos, size.width as i32, size.height as i32));
                }
            }
        }
    }

    rects
}

/// Tính toạ độ khởi tạo cho Bubble mới sao cho KHÔNG BAO GIỜ bị đè lên Bubble hiện có
pub fn calculate_stacked_bubble_pos(
    screen: &ScreenMetrics,
    config: &BubbleConfig,
    existing_bubbles: &[(PhysicalPosition<i32>, i32, i32)],
) -> PhysicalPosition<i32> {
    let scale = screen.scale_factor;
    let bubble_size = (config.size * scale) as i32;
    let gap = (12.0 * scale) as i32;
    let margin = (config.margin * scale) as i32;
    let taskbar_margin = (config.taskbar_margin * scale) as i32;
    let min_y = screen.y + margin;
    let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);

    let target_x = calculate_bubble_edge_x(screen, screen.x + screen.width, config.tuck_ratio, config);

    if existing_bubbles.is_empty() {
        return calculate_bubble_default_pos(screen, config);
    }

    // Lọc các bubble nằm ở cùng mép màn hình
    let same_edge_bubbles: Vec<_> = existing_bubbles
        .iter()
        .filter(|(pos, _, _)| (pos.x - target_x).abs() < bubble_size * 2)
        .collect();

    if same_edge_bubbles.is_empty() {
        return calculate_bubble_default_pos(screen, config);
    }

    // Tìm vị trí Y bên dưới bubble thấp nhất
    let mut candidate_y = min_y;
    if let Some(lowest) = same_edge_bubbles.iter().max_by_key(|(pos, _, h)| pos.y + h) {
        let next_y = lowest.0.y + lowest.2 + gap;
        if next_y <= max_y {
            candidate_y = next_y;
        } else if let Some(highest) = same_edge_bubbles.iter().min_by_key(|(pos, _, _)| pos.y) {
            let prev_y = highest.0.y - bubble_size - gap;
            if prev_y >= min_y {
                candidate_y = prev_y;
            }
        }
    }

    // Kiểm tra quét va chạm và dịch chuyển nếu có trùng lặp
    for (pos, _, h) in &same_edge_bubbles {
        if (candidate_y - pos.y).abs() < bubble_size + gap {
            candidate_y = (pos.y + h + gap).clamp(min_y, max_y);
        }
    }

    PhysicalPosition {
        x: target_x,
        y: candidate_y.clamp(min_y, max_y),
    }
}

/// Tính toạ độ hít mép có cơ chế chống va chạm (Collision Avoidance) giữa các Bubble
pub fn calculate_bubble_snap_with_collision(
    screen: &ScreenMetrics,
    win_pos: PhysicalPosition<i32>,
    config: &BubbleConfig,
    other_bubbles: &[(PhysicalPosition<i32>, i32, i32)],
) -> SnapResult {
    let scale = screen.scale_factor;
    let bubble_size = (config.size * scale) as i32;
    let gap = (10.0 * scale) as i32;
    let margin = (config.margin * scale) as i32;
    let taskbar_margin = (config.taskbar_margin * scale) as i32;

    // 1. Kiểm tra kéo sát đáy màn hình để ẩn (Dismiss)
    let bottom_threshold = screen.y + screen.height - (config.dismiss_margin * scale) as i32;
    if win_pos.y >= bottom_threshold {
        return SnapResult {
            target_x: win_pos.x,
            target_y: win_pos.y,
            should_dismiss: true,
        };
    }

    // 2. Tính target_x hít mép trái/phải
    let target_x = calculate_bubble_edge_x(screen, win_pos.x, config.tuck_ratio, config);

    let min_y = screen.y + margin;
    let max_y = (screen.y + screen.height - bubble_size - taskbar_margin).max(min_y);
    let mut candidate_y = win_pos.y.clamp(min_y, max_y);

    // 3. Tránh va chạm với các bubble khác ở cùng mép màn hình
    let same_edge_bubbles: Vec<_> = other_bubbles
        .iter()
        .filter(|(pos, _, _)| (pos.x - target_x).abs() < bubble_size * 2)
        .collect();

    for (pos, _, h) in &same_edge_bubbles {
        let other_top = pos.y;
        let other_bottom = pos.y + h;
        if (candidate_y - other_top).abs() < bubble_size + gap {
            if candidate_y >= other_top {
                candidate_y = (other_bottom + gap).min(max_y);
            } else {
                candidate_y = (other_top - bubble_size - gap).max(min_y);
            }
        }
    }

    SnapResult {
        target_x,
        target_y: candidate_y.clamp(min_y, max_y),
        should_dismiss: false,
    }
}

/// Hiển thị và định vị Bubble cho tài khoản chỉ định với cơ chế xếp chống va chạm
pub fn ensure_bubble_positioned_and_visible(app: &AppHandle, account_index: u32) -> Result<(), String> {
    let bubble_window = get_or_create_bubble_window(app, account_index)?;
    let win_label = bubble_window.label().to_string();

    let monitor = app
        .get_webview_window("main")
        .and_then(|m| {
            if m.is_visible().unwrap_or(false) {
                m.current_monitor().ok().flatten()
            } else {
                None
            }
        })
        .or_else(|| bubble_window.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let win_pos = bubble_window.outer_position().unwrap_or_default();
        let screen = ScreenMetrics::from_monitor(&monitor);
        let bubble_w = (DEFAULT_BUBBLE_CONFIG.size * screen.scale_factor) as i32;

        let is_visible = bubble_window.is_visible().unwrap_or(false);
        let is_out_of_bounds = win_pos.x < screen.x - bubble_w
            || win_pos.x > screen.x + screen.width
            || (win_pos.x == 0 && win_pos.y == 0);

        if !is_visible || is_out_of_bounds {
            let other_bubbles = get_other_visible_bubble_rects(app, Some(&win_label));
            let target_pos = calculate_stacked_bubble_pos(&screen, &DEFAULT_BUBBLE_CONFIG, &other_bubbles);
            let _ = bubble_window.set_position(tauri::Position::Physical(target_pos));
        }
    }
    let _ = bubble_window.show();
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": account_index,
        "visible": true
    }));
    Ok(())
}

/// Dock cửa sổ chat Messenger vào đúng Bubble của tài khoản tương ứng
pub fn dock_chat_window(app: &AppHandle, chat_window: &tauri::WebviewWindow, account_index: u32) {
    let bubble_label = get_bubble_window_label(account_index);
    let bubble_win = app.get_webview_window(&bubble_label);

    if let Some(bubble_window) = bubble_win {
        if let Ok(Some(monitor)) = bubble_window.current_monitor() {
            let screen = ScreenMetrics::from_monitor(&monitor);
            let scale = screen.scale_factor;

            let chat_w = (DEFAULT_BUBBLE_CONFIG.chat_width * scale).round() as i32;
            let chat_h = (DEFAULT_BUBBLE_CONFIG.chat_height * scale).round() as i32;
            let bubble_w = (DEFAULT_BUBBLE_CONFIG.size * scale).round() as i32;
            let gap = (DEFAULT_BUBBLE_CONFIG.chat_gap * scale).round() as i32;
            let margin = (DEFAULT_BUBBLE_CONFIG.margin * scale).round() as i32;
            let taskbar_margin = (DEFAULT_BUBBLE_CONFIG.taskbar_margin * scale).round() as i32;

            let _ = chat_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: chat_w as u32,
                height: chat_h as u32,
            }));

            if let Ok(bubble_pos) = bubble_window.outer_position() {
                let untucked_pos = calculate_bubble_hover_pos(
                    &screen,
                    bubble_pos.x,
                    bubble_pos.y,
                    true,
                    &DEFAULT_BUBBLE_CONFIG,
                );
                let _ = bubble_window.set_position(tauri::Position::Physical(untucked_pos));

                let mid_x = screen.x + screen.width / 2;

                let target_x = if untucked_pos.x + bubble_w / 2 >= mid_x {
                    untucked_pos.x - chat_w - gap
                } else {
                    untucked_pos.x + bubble_w + gap
                };

                let min_x = screen.x + margin;
                let max_x = screen.x + screen.width - chat_w - margin;
                let final_x = target_x.clamp(min_x, max_x);

                let min_y = screen.y + margin;
                let max_y = screen.y + screen.height - chat_h - taskbar_margin;
                let final_y = untucked_pos.y.clamp(min_y, max_y);

                let _ = chat_window.set_position(tauri::Position::Physical(PhysicalPosition {
                    x: final_x,
                    y: final_y,
                }));
            }
        }
    }
}

/// Tự động thụt các bong bóng nổi vào mép màn hình (tuck_ratio)
pub fn tuck_bubble(app: &AppHandle, account_index: Option<u32>) {
    let indices: Vec<u32> = match account_index {
        Some(idx) => vec![idx],
        None => (1..=5).collect(),
    };

    for idx in indices {
        let label = get_bubble_window_label(idx);
        let win = app.get_webview_window(&label);

        if let Some(bubble_window) = win {
            if let Ok(Some(monitor)) = bubble_window.current_monitor() {
                let screen = ScreenMetrics::from_monitor(&monitor);
                let win_pos = bubble_window.outer_position().unwrap_or_default();
                let tucked_pos = calculate_bubble_hover_pos(
                    &screen,
                    win_pos.x,
                    win_pos.y,
                    false,
                    &DEFAULT_BUBBLE_CONFIG,
                );
                let _ = bubble_window.set_position(tauri::Position::Physical(tucked_pos));
            }
        }
    }
}

/// Kiểm tra xem con trỏ chuột hiện tại có đang nằm trên bất kỳ cửa sổ bong bóng nổi nào không
pub fn is_cursor_over_any_bubble(app: &AppHandle) -> bool {
    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        let win = app.get_webview_window(&label);

        if let Some(w) = win {
            if w.is_visible().unwrap_or(false) && is_cursor_over_bubble(&w) {
                return true;
            }
        }
    }
    false
}

/// Kiểm tra xem con trỏ chuột hiện tại có đang nằm trên cửa sổ bong bóng nổi hay không
#[cfg(target_os = "windows")]
pub fn is_cursor_over_bubble(bubble_window: &tauri::WebviewWindow) -> bool {
    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    }

    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut pt) == 0 {
            return false;
        }
    }

    if let Ok(pos) = bubble_window.outer_position() {
        if let Ok(size) = bubble_window.outer_size() {
            return pt.x >= pos.x
                && pt.x <= pos.x + size.width as i32
                && pt.y >= pos.y
                && pt.y <= pos.y + size.height as i32;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn is_cursor_over_bubble(_bubble_window: &tauri::WebviewWindow) -> bool {
    false
}
