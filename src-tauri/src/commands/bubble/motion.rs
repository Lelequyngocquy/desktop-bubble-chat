use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};

use crate::movement::{
    get_bubble_physics_config, BubbleConfigState, BubbleMovementEngine, BubbleRect,
};
use crate::state::ChatState;
use crate::presentation::bubble::{get_active_bubble_rects, get_bubble_window_label, resize_bubble_window};

/// Snaps bubble to screen edge with collision avoidance
#[tauri::command]
pub fn snap_bubble(app: AppHandle, account_index: Option<u32>) -> Result<(), String> {
    let config = get_bubble_physics_config(&app);
    if config.is_free_position {
        return Ok(());
    }
    let idx = account_index.unwrap_or(1);
    if crate::operating_systems::current::bubble::NativeBubbleManager::global().has_bubble(idx) {
        if let Some((cur_x, cur_y, _, _)) = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(idx) {
            let screen = crate::movement::get_target_dashboard_screen(Some(&app));
            let config = get_bubble_physics_config(&app);
            let all_bubbles = get_active_bubble_rects(&app, None);
            let snap = BubbleMovementEngine::calculate_snap(
                &screen,
                &config,
                PhysicalPosition { x: cur_x, y: cur_y },
                idx,
                &all_bubbles,
            );
            if snap.should_dismiss {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = super::lifecycle::toggle_bubble(app_clone, false, Some(idx)).await;
                });
                return Ok(());
            }
            crate::operating_systems::current::bubble::NativeBubbleManager::global().dispatch(
                crate::operating_systems::current::bubble::NativeBubbleAction::AnimateTo {
                    account_index: idx,
                    target_x: snap.target_x,
                    target_y: snap.target_y,
                    duration_ms: 160,
                }
            );
            return Ok(());
        }
    }
    let label = get_bubble_window_label(idx);
    if let Some(window) = app.get_webview_window(&label) {
        let win_pos = window.outer_position().map_err(|e| e.to_string())?;
        let screen = crate::movement::get_target_dashboard_screen(Some(&app));
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
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = super::lifecycle::toggle_bubble(app_clone, false, Some(idx)).await;
            });
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
    Ok(())
}

/// Updates bubble physics configuration (size, tuck ratio)
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
    if let Some(s) = size {
        let size_str = if s <= 48.0 {
            "sm".to_string()
        } else if s >= 64.0 {
            "lg".to_string()
        } else {
            "md".to_string()
        };
        if let Some(chat_state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
            chat_state.update_user_config(&app, |c| {
                c.bubble_size = size_str;
            });
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
    let screen = crate::movement::get_target_dashboard_screen(Some(&app));
    let bubble_size_px = (config.size * screen.scale_factor).round() as u32;
    for idx in 1..=5 {
        if crate::operating_systems::current::bubble::has_native_bubble(idx) {
            crate::operating_systems::current::bubble::set_native_bubble_size(idx, bubble_size_px);
            if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
                if let Some(mut rect) = chat_state.get_bubble_rect(idx) {
                    rect.width = bubble_size_px as i32;
                    rect.height = bubble_size_px as i32;
                    chat_state.set_bubble_rect(idx, rect);
                }
            }
            let _ = snap_bubble(app.clone(), Some(idx));
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

/// Sets bubble position mode: "auto_tuck" (default) or "free"
#[tauri::command]
pub fn set_bubble_position_mode(
    app: AppHandle,
    mode: String,
) -> Result<(), String> {
    let is_free = mode == "free";
    if let Some(state) = app.try_state::<BubbleConfigState>() {
        if let Ok(mut guard) = state.0.write() {
            guard.is_free_position = is_free;
        }
    }
    if let Some(chat_state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
        chat_state.update_user_config(&app, |c| {
            c.bubble_position_mode = mode.clone();
        });
    }
    let _ = app.emit(
        "bubble-config-changed",
        serde_json::json!({
            "bubblePositionMode": mode,
        }),
    );
    Ok(())
}
