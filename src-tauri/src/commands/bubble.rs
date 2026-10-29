use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};
use crate::utils::positioning::{
    calculate_bubble_default_pos, calculate_bubble_hover_pos, calculate_bubble_snap,
    ensure_bubble_positioned_and_visible, ScreenMetrics, DEFAULT_BUBBLE_CONFIG,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapCoordsDto {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
}

/// Command điều khiển ẩn/hiện cửa sổ bong bóng nổi
#[tauri::command]
pub fn toggle_bubble(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(bubble_window) = app.get_webview_window("bubble") {
        if visible {
            let _ = ensure_bubble_positioned_and_visible(&app);
        } else {
            let _ = bubble_window.hide();
            let _ = app.emit("bubble-visibility-changed", false);
        }
    }
    Ok(())
}

/// Command tự động hít vào mép màn hình (Snap to Edge) & Dismiss đáy
#[tauri::command]
pub fn snap_bubble(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("bubble") {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let win_pos = window.outer_position().map_err(|e| e.to_string())?;
            let screen = ScreenMetrics::from_monitor(&monitor);
            let snap = calculate_bubble_snap(&screen, win_pos, &DEFAULT_BUBBLE_CONFIG);

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

/// Command cung cấp tọa độ hít mép cho Frontend (Single Source of Truth)
#[tauri::command]
pub fn get_bubble_snap_coords(
    app: AppHandle,
    current_x: i32,
    current_y: i32,
) -> Result<SnapCoordsDto, String> {
    if let Some(window) = app.get_webview_window("bubble") {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen = ScreenMetrics::from_monitor(&monitor);
            let win_pos = PhysicalPosition {
                x: current_x,
                y: current_y,
            };
            let snap = calculate_bubble_snap(&screen, win_pos, &DEFAULT_BUBBLE_CONFIG);
            return Ok(SnapCoordsDto {
                target_x: snap.target_x,
                target_y: snap.target_y,
                should_dismiss: snap.should_dismiss,
            });
        }
    }
    Err("Monitor not available".into())
}

/// Command cung cấp tọa độ mặc định của bong bóng nổi (mép phải, 1/3 chiều cao từ trên xuống)
#[tauri::command]
pub fn get_bubble_default_coords(app: AppHandle) -> Result<SnapCoordsDto, String> {
    if let Some(window) = app.get_webview_window("bubble") {
        let monitor = app
            .get_webview_window("main")
            .and_then(|m| {
                if m.is_visible().unwrap_or(false) {
                    m.current_monitor().ok().flatten()
                } else {
                    None
                }
            })
            .or_else(|| window.current_monitor().ok().flatten())
            .or_else(|| app.primary_monitor().ok().flatten());

        if let Some(monitor) = monitor {
            let screen = ScreenMetrics::from_monitor(&monitor);
            let default_pos = calculate_bubble_default_pos(&screen, &DEFAULT_BUBBLE_CONFIG);
            return Ok(SnapCoordsDto {
                target_x: default_pos.x,
                target_y: default_pos.y,
                should_dismiss: false,
            });
        }
    }
    Err("Monitor not available".into())
}

/// Command lấy tọa độ khi hover vào/ra khỏi bong bóng (appear_ratio 0.0 vs tuck_ratio)
#[tauri::command]
pub fn get_bubble_hover_coords(
    app: AppHandle,
    is_hovered: bool,
) -> Result<SnapCoordsDto, String> {
    if let Some(window) = app.get_webview_window("bubble") {
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
