use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};
use crate::utils::positioning::{
    calculate_bubble_hover_pos, calculate_bubble_snap_with_collision,
    calculate_stacked_bubble_pos, ensure_bubble_positioned_and_visible,
    get_bubble_window_label, get_other_visible_bubble_rects, ScreenMetrics, DEFAULT_BUBBLE_CONFIG,
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
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

    if let Some(w) = win {
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
        let win = app.get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

        if let Some(w) = win {
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
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

    if visible {
        let _ = ensure_bubble_positioned_and_visible(&app, idx);
    } else if let Some(bubble_window) = win {
        let _ = bubble_window.hide();
        let _ = app.emit("bubble-visibility-changed", false);
    }
    Ok(())
}

/// Command tự động hít vào mép màn hình (Snap to Edge) có cơ chế tránh va chạm giữa các Bubble
#[tauri::command]
pub fn snap_bubble(app: AppHandle, account_index: Option<u32>) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let label = get_bubble_window_label(idx);
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

    if let Some(window) = win {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let win_pos = window.outer_position().map_err(|e| e.to_string())?;
            let screen = ScreenMetrics::from_monitor(&monitor);
            let other_bubbles = get_other_visible_bubble_rects(&app, Some(window.label()));
            let snap = calculate_bubble_snap_with_collision(&screen, win_pos, &DEFAULT_BUBBLE_CONFIG, &other_bubbles);

            if snap.should_dismiss {
                let _ = window.hide();
                let _ = app.emit("bubble-visibility-changed", false);
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
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

    if let Some(window) = win {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen = ScreenMetrics::from_monitor(&monitor);
            let win_pos = PhysicalPosition {
                x: current_x,
                y: current_y,
            };
            let other_bubbles = get_other_visible_bubble_rects(&app, Some(window.label()));
            let snap = calculate_bubble_snap_with_collision(&screen, win_pos, &DEFAULT_BUBBLE_CONFIG, &other_bubbles);
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
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

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
        let exclude_label = win.as_ref().map(|w| w.label());
        let other_bubbles = get_other_visible_bubble_rects(&app, exclude_label);
        let pos = calculate_stacked_bubble_pos(&screen, &DEFAULT_BUBBLE_CONFIG, &other_bubbles);
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
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None });

    if let Some(window) = win {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let win_pos = window.outer_position().unwrap_or_default();
            let screen = ScreenMetrics::from_monitor(&monitor);
            let target_pos = calculate_bubble_hover_pos(
                &screen,
                win_pos.x,
                win_pos.y,
                is_hovered,
                &DEFAULT_BUBBLE_CONFIG,
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