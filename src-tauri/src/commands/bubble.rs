use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};

use crate::movement::{
    get_bubble_physics_config, BubbleConfigState, BubbleMovementEngine, BubbleRect, ScreenMetrics,
};
use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::ChatState;
use crate::windows::bubble::{
    ensure_bubble_positioned_and_visible, ensure_bubble_positioned_and_visible_with_extra,
    get_active_bubble_rects, get_bubble_window_label, resize_bubble_window,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapCoordsDto {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
    pub is_left_edge: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BubblePositionDto {
    pub account_index: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_visible: bool,
}

/// Command lấy vị trí hiện tại của một Bubble chỉ định
#[tauri::command]
pub fn get_bubble_position(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<PhysicalPosition<i32>, String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);

    if let Some(w) = app.get_webview_window(&label) {
        return w.outer_position().map_err(|e| e.to_string());
    }
    Err(format!("Bubble for account {} not found", idx))
}

/// Command lấy danh sách tọa độ của tất cả các Bubble trên màn hình
#[tauri::command]
pub fn get_all_bubble_positions(app: AppHandle) -> Result<Vec<BubblePositionDto>, String> {
    let mut list = Vec::new();
    let state_rects: std::collections::HashMap<u32, BubbleRect> = app
        .try_state::<Arc<ChatState>>()
        .and_then(|s| s.active_bubble_rects.lock().ok().map(|g| g.clone()))
        .unwrap_or_default();

    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(w) = app.get_webview_window(&label) {
            let is_vis = w.is_visible().unwrap_or(false);
            let mut pos = w.outer_position().unwrap_or_default();
            let mut size = w.outer_size().unwrap_or_default();

            if pos.x == 0 && pos.y == 0 {
                if let Some(sr) = state_rects.get(&idx) {
                    pos.x = sr.x;
                    pos.y = sr.y;
                    size.width = sr.width as u32;
                    size.height = sr.height as u32;
                }
            }

            list.push(BubblePositionDto {
                account_index: idx,
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
                is_visible: is_vis,
            });
        }
    }
    Ok(list)
}

/// Helper shutdown: Hủy cửa sổ chat webview, giải phóng RAM, terminate process nhưng bảo toàn cache trên đĩa
pub fn suspend_account_bubble_and_chat(app: &AppHandle, account_index: u32) {
    let idx = account_index;
    let bubble_label = get_bubble_window_label(idx);
    let chat_label = get_messenger_window_label(idx);

    if let Some(state) = app.try_state::<Arc<ChatState>>() {
        state.set_bypass_close_guard(&chat_label, true);
        state.set_account_suspended(idx, true);
        state.remove_bubble_rect(idx);
    }

    // 1. Hủy cửa sổ chat Messenger (giải phóng WebView2 COM/DOM trong RAM)
    let win = app
        .get_webview_window(&chat_label)
        .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

    if let Some(cw) = win {
        let _ = cw.hide();
        let _ = cw.close();
        let _ = cw.destroy();
    }

    // 2. Ẩn và tiêu hủy cửa sổ Bubble (Dynamic Destruction giải phóng Renderer RAM của Bong bóng)
    if let Some(bw) = app.get_webview_window(&bubble_label) {
        let _ = bw.hide();
        let _ = bw.destroy();
    }

    #[cfg(target_os = "windows")]
    crate::windows::main_window::trim_current_process_working_set();

    // 4. Phát event thông báo đã ẩn
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": idx,
        "visible": false
    }));
}

/// Helper resume: Tái sử dụng flow của Bubble 05 (nạp lại webview từ cache đĩa và đặt vị trí snap)
pub fn resume_account_bubble_and_chat(
    app: &AppHandle,
    account_index: u32,
    committed_rects: Option<&[BubbleRect]>,
) -> Result<BubbleRect, String> {
    let idx = account_index;

    // 1. Đánh dấu gỡ trạng thái ngủ đông trong state
    if let Some(state) = app.try_state::<Arc<ChatState>>() {
        state.set_account_suspended(idx, false);
    }

    // 2. Định vị và hiển thị Bubble lên màn hình ngay lập tức (Hàm chuẩn của Bubble 05).
    // ensure_bubble_positioned_and_visible đã tích hợp sẵn preload_messenger_window
    // chạy ngầm (async spawn) để tải session chat từ cache đĩa mà không block UI.
    let rect = if let Some(committed) = committed_rects {
        ensure_bubble_positioned_and_visible_with_extra(app, idx, committed)?
    } else {
        ensure_bubble_positioned_and_visible(app, idx)?
    };

    // 3. Phát event thông báo
    let _ = app.emit("bubble-spawned", serde_json::json!({
        "accountIndex": idx,
        "x": rect.x,
        "y": rect.y
    }));
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": idx,
        "visible": true
    }));

    Ok(rect)
}

/// RAII Guard bảo đảm khóa transition_lock luôn luôn được giải phóng khi kết thúc tác vụ
struct TransitionLockGuard {
    state: Option<Arc<ChatState>>,
    account_index: u32,
}

impl TransitionLockGuard {
    fn new(state: Option<Arc<ChatState>>, account_index: u32) -> Self {
        Self { state, account_index }
    }
}

impl Drop for TransitionLockGuard {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            state.release_transition_lock(self.account_index);
        }
    }
}

/// Command điều khiển ẩn/hiện cửa sổ bong bóng nổi cho tài khoản chỉ định
#[tauri::command]
pub async fn toggle_bubble(
    app: AppHandle,
    visible: bool,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);

    let state = app.try_state::<Arc<ChatState>>().map(|s| Arc::clone(&s));
    if let Some(s) = &state {
        if !s.try_acquire_transition_lock(idx) {
            println!("[toggle_bubble] Account {} is transitioning, skipping duplicate toggle", idx);
            return Ok(());
        }
    }
    let guard = TransitionLockGuard::new(state, idx);

    if visible {
        match resume_account_bubble_and_chat(&app, idx, None) {
            Ok(_) => println!("[toggle_bubble] Account {} bubble resumed successfully", idx),
            Err(e) => eprintln!("[toggle_bubble ERROR] Failed to resume bubble for account {}: {}", idx, e),
        }
    } else {
        suspend_account_bubble_and_chat(&app, idx);
        println!("[toggle_bubble] Account {} bubble suspended successfully", idx);
    }

    drop(guard);

    let _ = crate::tray::update_tray_menu(&app);

    Ok(())
}

/// Command tự động hít vào mép màn hình (Snap to Edge) có cơ chế tránh va chạm giữa các Bubble
#[tauri::command]
pub fn snap_bubble(app: AppHandle, account_index: Option<u32>) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);

    if let Some(window) = app.get_webview_window(&label) {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let win_pos = window.outer_position().map_err(|e| e.to_string())?;
            let screen = ScreenMetrics::from_monitor(&monitor);
            let config = get_bubble_physics_config(&app);
            let all_bubbles = get_active_bubble_rects(&app, None);

            let snap = BubbleMovementEngine::calculate_snap(
                &screen,
                &config,
                win_pos,
                idx,
                &all_bubbles,
            );

            if snap.should_dismiss {
                suspend_account_bubble_and_chat(&app, idx);
                return Ok(());
            }

            window
                .set_position(Position::Physical(PhysicalPosition {
                    x: snap.target_x,
                    y: snap.target_y,
                }))
                .map_err(|e| e.to_string())?;

            if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
                let bubble_w = (config.size * screen.scale_factor) as i32;
                chat_state.set_bubble_rect(
                    idx,
                    BubbleRect {
                        account_index: idx,
                        x: snap.target_x,
                        y: snap.target_y,
                        width: bubble_w,
                        height: bubble_w,
                    },
                );
            }
        }
    }
    Ok(())
}

/// Command cung cấp tọa độ hít mép cho Frontend (SSOT) đảm bảo không bao giờ đè lên Bubble khác
#[tauri::command]
pub fn get_bubble_snap_coords(
    app: AppHandle,
    current_x: i32,
    current_y: i32,
    account_index: Option<u32>,
) -> Result<SnapCoordsDto, String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);

    if let Some(window) = app.get_webview_window(&label) {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen = ScreenMetrics::from_monitor(&monitor);
            let win_pos = PhysicalPosition {
                x: current_x,
                y: current_y,
            };
            let config = get_bubble_physics_config(&app);
            let all_bubbles = get_active_bubble_rects(&app, None);

            let snap = BubbleMovementEngine::calculate_snap(
                &screen,
                &config,
                win_pos,
                idx,
                &all_bubbles,
            );

            return Ok(SnapCoordsDto {
                target_x: snap.target_x,
                target_y: snap.target_y,
                should_dismiss: snap.should_dismiss,
                is_left_edge: snap.is_left_edge,
            });
        }
    }
    Err("Monitor not available".into())
}

/// Command cung cấp tọa độ mặc định của bong bóng nổi (có kiểm tra chống va chạm với Bubble đã có)
#[tauri::command]
pub fn get_bubble_default_coords(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<SnapCoordsDto, String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);
    let win = app.get_webview_window(&label);

    let monitor = app
        .get_webview_window("main")
        .and_then(|m| {
            if m.is_visible().unwrap_or(false) {
                m.current_monitor().ok().flatten()
            } else {
                None
            }
        })
        .or_else(|| win.as_ref().and_then(|w| w.current_monitor().ok().flatten()))
        .or_else(|| app.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let screen = ScreenMetrics::from_monitor(&monitor);
        let config = get_bubble_physics_config(&app);
        let all_bubbles = get_active_bubble_rects(&app, None);

        let pos = BubbleMovementEngine::calculate_stacked_pos(&screen, &config, &all_bubbles);
        return Ok(SnapCoordsDto {
            target_x: pos.x,
            target_y: pos.y,
            should_dismiss: false,
            is_left_edge: true,
        });
    }
    Err("Monitor not available".into())
}

/// Command lấy tọa độ khi hover vào/ra khỏi bong bóng (appear_ratio 0.0 vs tuck_ratio)
#[tauri::command]
pub fn get_bubble_hover_coords(
    app: AppHandle,
    is_hovered: bool,
    account_index: Option<u32>,
) -> Result<SnapCoordsDto, String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);

    if let Some(window) = app.get_webview_window(&label) {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let win_pos = match window.outer_position() {
                Ok(pos) => pos,
                Err(_) => return Err("Failed to get window position".into()),
            };
            let screen = ScreenMetrics::from_monitor(&monitor);
            let config = get_bubble_physics_config(&app);

            let target_pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                win_pos.x,
                win_pos.y,
                is_hovered,
            );
            let bubble_w = (config.size * screen.scale_factor) as i32;
            let is_left = win_pos.x + bubble_w / 2 < (screen.x + screen.width / 2);

            return Ok(SnapCoordsDto {
                target_x: target_pos.x,
                target_y: target_pos.y,
                should_dismiss: false,
                is_left_edge: is_left,
            });
        }
    }
    Err("Monitor not available".into())
}

/// Command cập nhật cấu hình vật lý của Bong bóng (kích thước, tỉ lệ lấn mép)
#[tauri::command]
pub fn set_bubble_config(
    app: AppHandle,
    size: Option<f64>,
    tuck_ratio: Option<f64>,
) -> Result<(), String> {
    if let Some(state) = app.try_state::<BubbleConfigState>() {
        if let Ok(mut guard) = state.0.write() {
            if let Some(s) = size {
                guard.size = s;
            }
            if let Some(r) = tuck_ratio {
                guard.tuck_ratio = r;
            }
        }
    }

    let config = get_bubble_physics_config(&app);
    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(window) = app.get_webview_window(&label) {
            resize_bubble_window(&window, config.size);
            if window.is_visible().unwrap_or(false) {
                let _ = snap_bubble(app.clone(), Some(idx));
            }
        }
    }

    let size_str = if config.size <= 50.0 {
        "sm"
    } else if config.size <= 60.0 {
        "md"
    } else {
        "lg"
    };

    let _ = app.emit(
        "bubble-config-changed",
        serde_json::json!({
            "size": config.size,
            "bubbleSize": size_str,
            "tuckRatio": config.tuck_ratio,
        }),
    );
    Ok(())
}

/// Helper & command bật/tắt toàn bộ bong bóng nổi tuần tự (Async Sequential Pipeline & Committed Slots)
pub async fn toggle_all_bubbles_impl(
    app: &AppHandle,
    force_visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> bool {
    let mut visible_indices: Vec<u32> = Vec::new();
    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(bw) = app.get_webview_window(&label) {
            if bw.is_visible().unwrap_or(false) {
                visible_indices.push(idx);
            }
        }
    }

    let any_visible = !visible_indices.is_empty();

    let next_visible = match force_visible {
        Some(v) => v,
        None => !any_visible,
    };

    if next_visible {
        let targets = if let Some(indices) = account_indices {
            indices
        } else {
            let available = crate::tray::get_available_account_indices(app);
            if available.is_empty() {
                vec![1]
            } else {
                available
            }
        };

        let mut committed_rects: Vec<BubbleRect> = Vec::new();

        for (i, idx) in targets.iter().enumerate() {
            match resume_account_bubble_and_chat(app, *idx, Some(&committed_rects)) {
                Ok(rect) => {
                    println!("[toggle_all_bubbles] Account {} bubble resumed successfully", idx);
                    committed_rects.push(rect);
                }
                Err(err) => {
                    eprintln!("[toggle_all_bubbles ERROR] Failed to resume bubble for account {}: {}", idx, err);
                }
            }
            // Giãn cách 70ms giữa các bubble để Windows DWM vẽ và tạo hiệu ứng thác nước mượt mà
            if i + 1 < targets.len() {
                tokio::time::sleep(std::time::Duration::from_millis(70)).await;
            }
        }
    } else {
        // Ghi nhớ danh sách các Bubble đang mở trước khi ẩn
        if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
            chat_state.save_previously_visible_bubbles(visible_indices);
        }

        for idx in 1..=5 {
            suspend_account_bubble_and_chat(app, idx);
        }
    }

    let _ = crate::tray::update_tray_menu(app);

    next_visible
}

#[tauri::command]
pub async fn toggle_all_bubbles(
    app: AppHandle,
    visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> Result<bool, String> {
    Ok(toggle_all_bubbles_impl(&app, visible, account_indices).await)
}
