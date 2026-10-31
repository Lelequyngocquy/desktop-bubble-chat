use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};

use crate::movement::{
    get_bubble_physics_config, BubbleConfigState, BubbleMovementEngine, BubbleRect, ScreenMetrics,
};
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
        if let Ok(rect) = ensure_bubble_positioned_and_visible(&app, idx) {
            let _ = app.emit("bubble-spawned", serde_json::json!({
                "accountIndex": idx,
                "x": rect.x,
                "y": rect.y
            }));
        }
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
        let chat_state = app.try_state::<Arc<ChatState>>();

        let targets = if let Some(indices) = account_indices {
            indices
        } else {
            // 1. Ưu tiên các Bubble đã hiển thị trước thời điểm ẩn (ghi nhớ trạng thái)
            let prev = chat_state
                .as_ref()
                .map(|s| s.get_previously_visible_bubbles())
                .unwrap_or_default();

            if !prev.is_empty() {
                prev
            } else {
                // 2. Nếu chưa có lịch sử, lấy tất cả tài khoản đã kết nối
                let mut connected = Vec::new();
                if let Some(state) = chat_state.as_ref() {
                    if let Ok(map) = state.account_auth.lock() {
                        for (idx, info) in map.iter() {
                            if info.connected {
                                connected.push(*idx);
                            }
                        }
                    }
                }
                connected.sort();

                if !connected.is_empty() {
                    connected
                } else {
                    // 3. Kiểm tra các profile tồn tại trên đĩa
                    let mut disk_profiles = Vec::new();
                    if let Ok(app_data_dir) = app.path().app_data_dir() {
                        let profiles_dir = app_data_dir.join("profiles");
                        for idx in 1..=5 {
                            if profiles_dir.join(format!("messenger_{:02}", idx)).exists() {
                                disk_profiles.push(idx);
                            }
                        }
                    }

                    if !disk_profiles.is_empty() {
                        disk_profiles
                    } else {
                        vec![1]
                    }
                }
            }
        };

        let mut committed_rects: Vec<BubbleRect> = Vec::new();

        for (i, idx) in targets.iter().enumerate() {
            if let Ok(rect) = ensure_bubble_positioned_and_visible_with_extra(app, *idx, &committed_rects) {
                committed_rects.push(rect);
                let _ = app.emit("bubble-spawned", serde_json::json!({
                    "accountIndex": *idx,
                    "x": rect.x,
                    "y": rect.y
                }));
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
pub async fn toggle_all_bubbles(
    app: AppHandle,
    visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> Result<bool, String> {
    Ok(toggle_all_bubbles_impl(&app, visible, account_indices).await)
}
