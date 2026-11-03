use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};

use crate::movement::{
    get_bubble_physics_config, BubbleConfigState, BubbleMovementEngine, BubbleRect, ScreenMetrics,
};
use crate::state::ChatState;
use crate::windows::bubble::{get_active_bubble_rects, get_bubble_window_label, resize_bubble_window};

/// Command tự động hít vào mép màn hình (Snap to Edge) có cơ chế tránh va chạm giữa các Bubble
#[tauri::command]
pub fn snap_bubble(app: AppHandle, account_index: Option<u32>) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);

    #[cfg(target_os = "windows")]
    {
        if crate::windows::native_bubble::NativeBubbleManager::global().has_bubble(idx) {
            let monitor = app
                .get_webview_window("main")
                .and_then(|m| m.current_monitor().ok().flatten())
                .or_else(|| app.primary_monitor().ok().flatten());

            if let Some(mon) = monitor {
                if let Some((cur_x, cur_y, _, _)) = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(idx) {
                    let screen = ScreenMetrics::from_monitor(&mon);
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
                        super::lifecycle::suspend_account_bubble_and_chat(&app, idx);
                        return Ok(());
                    }

                    crate::windows::native_bubble::NativeBubbleManager::global().dispatch(
                        crate::windows::native_bubble::NativeBubbleAction::AnimateTo {
                            account_index: idx,
                            target_x: snap.target_x,
                            target_y: snap.target_y,
                            duration_ms: 160,
                        }
                    );
                    return Ok(());
                }
            }
        }
    }

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
                super::lifecycle::suspend_account_bubble_and_chat(&app, idx);
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

    #[cfg(target_os = "windows")]
    {
        let monitor = app
            .get_webview_window("main")
            .and_then(|m| m.current_monitor().ok().flatten())
            .or_else(|| app.primary_monitor().ok().flatten());
        let scale = monitor.map(|m| m.scale_factor()).unwrap_or(1.0);
        let bubble_size_px = (config.size * scale).round() as u32;

        for idx in 1..=5 {
            if crate::windows::native_bubble::has_native_bubble(idx) {
                crate::windows::native_bubble::set_native_bubble_size(idx, bubble_size_px);
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
