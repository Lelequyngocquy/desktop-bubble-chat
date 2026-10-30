use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};

use crate::movement::{
    get_bubble_physics_config, BubbleConfigState, BubbleMovementEngine, ScreenMetrics,
};
use crate::windows::bubble::{
    ensure_bubble_positioned_and_visible, get_active_bubble_rects, get_bubble_window_label,
    resize_bubble_window,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapCoordsDto {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
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
    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(w) = app.get_webview_window(&label) {
            let is_vis = w.is_visible().unwrap_or(false);
            let pos = w.outer_position().unwrap_or_default();
            let size = w.outer_size().unwrap_or_default();
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

/// Command điều khiển ẩn/hiện cửa sổ bong bóng nổi cho tài khoản chỉ định
#[tauri::command]
pub fn toggle_bubble(
    app: AppHandle,
    visible: bool,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);

    if visible {
        let _ = ensure_bubble_positioned_and_visible(&app, idx);
    } else if let Some(bubble_window) = app.get_webview_window(&label) {
        let _ = bubble_window.hide();
        let _ = app.emit("bubble-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": false
        }));
    }
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
                let _ = window.hide();
                let _ = app.emit("bubble-visibility-changed", serde_json::json!({
                    "accountIndex": idx,
                    "visible": false
                }));
                return Ok(());
            }

            window
                .set_position(Position::Physical(PhysicalPosition {
                    x: snap.target_x,
                    y: snap.target_y,
                }))
                .map_err(|e| e.to_string())?;
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
            let win_pos = window.outer_position().unwrap_or_default();
            let screen = ScreenMetrics::from_monitor(&monitor);
            let config = get_bubble_physics_config(&app);

            let target_pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                win_pos.x,
                win_pos.y,
                is_hovered,
            );
            return Ok(SnapCoordsDto {
                target_x: target_pos.x,
                target_y: target_pos.y,
                should_dismiss: false,
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

/// Helper & command bật/tắt toàn bộ bong bóng nổi (dùng chung cho Tray, Shortcut và IPC)
pub fn toggle_all_bubbles_impl(app: &AppHandle, force_visible: Option<bool>) -> bool {
    let mut any_visible = false;
    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(bw) = app.get_webview_window(&label) {
            if bw.is_visible().unwrap_or(false) {
                any_visible = true;
                break;
            }
        }
    }

    let next_visible = match force_visible {
        Some(v) => v,
        None => !any_visible,
    };

    if next_visible {
        let _ = ensure_bubble_positioned_and_visible(app, 1);
    } else {
        for idx in 1..=5 {
            let label = get_bubble_window_label(idx);
            if let Some(bw) = app.get_webview_window(&label) {
                let _ = bw.hide();
                let _ = app.emit("bubble-visibility-changed", serde_json::json!({
                    "accountIndex": idx,
                    "visible": false
                }));
            }
        }
    }

    next_visible
}

#[tauri::command]
pub fn toggle_all_bubbles(app: AppHandle, visible: Option<bool>) -> Result<bool, String> {
    Ok(toggle_all_bubbles_impl(&app, visible))
}
