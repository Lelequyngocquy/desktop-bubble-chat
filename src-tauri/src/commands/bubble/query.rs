use std::sync::Arc;
use tauri::{AppHandle, Manager, PhysicalPosition};

use super::dto::{BubblePositionDto, SnapCoordsDto};
use crate::movement::{get_bubble_physics_config, BubbleMovementEngine, BubbleRect, ScreenMetrics};
use crate::state::ChatState;
use crate::windows::bubble::{get_active_bubble_rects, get_bubble_window_label};

/// Command lấy vị trí hiện tại của một Bubble chỉ định
#[tauri::command]
pub fn get_bubble_position(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<PhysicalPosition<i32>, String> {
    let idx = account_index.unwrap_or(1);

    #[cfg(target_os = "windows")]
    {
        if let Some((x, y, _, _)) = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(idx) {
            return Ok(PhysicalPosition { x, y });
        }
    }

    if let Some(cs) = app.try_state::<Arc<ChatState>>() {
        if let Some(rect) = cs.get_bubble_rect(idx) {
            return Ok(PhysicalPosition { x: rect.x, y: rect.y });
        }
    }

    let label = get_bubble_window_label(idx);
    if let Some(w) = app.get_webview_window(&label) {
        return w.outer_position().map_err(|e| e.to_string());
    }
    Err(format!("Bubble for account {} not found", idx))
}

/// Command lấy danh sách các account_index đang có Bubble hiển thị (Active) trên màn hình (Read-Only SSOT)
#[tauri::command]
pub fn get_active_bubble_accounts(app: AppHandle) -> Result<Vec<u32>, String> {
    let mut active = Vec::new();
    for idx in 1..=5 {
        let is_suspended = app
            .try_state::<Arc<ChatState>>()
            .map(|s| s.is_account_suspended(idx))
            .unwrap_or(false);

        if !is_suspended && crate::tray::is_bubble_visible(&app, idx) {
            active.push(idx);
        }
    }
    Ok(active)
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
        #[cfg(target_os = "windows")]
        {
            if crate::windows::native_bubble::NativeBubbleManager::global().has_bubble(idx) {
                if let Some((x, y, w, h)) = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(idx) {
                    list.push(BubblePositionDto {
                        account_index: idx,
                        x,
                        y,
                        width: w,
                        height: h,
                        is_visible: true,
                    });
                    continue;
                }
            }
        }

        if let Some(sr) = state_rects.get(&idx) {
            let is_suspended = app
                .try_state::<Arc<ChatState>>()
                .map(|s| s.is_account_suspended(idx))
                .unwrap_or(false);

            if !is_suspended {
                list.push(BubblePositionDto {
                    account_index: idx,
                    x: sr.x,
                    y: sr.y,
                    width: sr.width as u32,
                    height: sr.height as u32,
                    is_visible: true,
                });
                continue;
            }
        }

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
