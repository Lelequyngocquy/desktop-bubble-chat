use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position};

use crate::movement::{
    get_bubble_physics_config, BubbleConfigState, BubbleMovementEngine, BubbleRect,
};
use crate::state::ChatState;
use crate::presentation::bubble::{get_active_bubble_rects, get_bubble_window_label, resize_bubble_window};

/// Classifies logical bubble diameter into canonical size preset label
pub fn classify_bubble_size(size: f64) -> &'static str {
    if size <= 48.0 {
        "sm"
    } else if size >= 64.0 {
        "lg"
    } else {
        "md"
    }
}

/// Snaps bubble to screen edge with collision avoidance
#[tauri::command]
pub fn snap_bubble(app: AppHandle, account_index: Option<u32>) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    let config = get_bubble_physics_config(&app);
    if config.is_free_position {
        return Ok(());
    }
    let stay_untucked = crate::presentation::bubble::is_bubble_stay_untucked(&app, idx);
    let edge_ratio = BubbleMovementEngine::effective_edge_ratio(&config, stay_untucked);
    if crate::operating_systems::current::bubble::NativeBubbleManager::global().has_bubble(idx)
        && let Some((cur_x, cur_y, _, _)) = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(idx) {
            let screen = crate::movement::get_target_dashboard_screen(Some(&app));
            let config = get_bubble_physics_config(&app);
            let all_bubbles = get_active_bubble_rects(&app, None);
            let snap = BubbleMovementEngine::calculate_snap_with_ratio(
                &screen,
                &config,
                PhysicalPosition { x: cur_x, y: cur_y },
                idx,
                &all_bubbles,
                edge_ratio,
            );
            if snap.should_dismiss {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    super::lifecycle::dismiss_bubble_from_drag(app_clone, idx).await;
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
    let label = get_bubble_window_label(idx);
    if let Some(window) = app.get_webview_window(&label) {
        let win_pos = window.outer_position().map_err(|e| e.to_string())?;
        let screen = crate::movement::get_target_dashboard_screen(Some(&app));
        let config = get_bubble_physics_config(&app);
        let all_bubbles = get_active_bubble_rects(&app, None);
        let snap = BubbleMovementEngine::calculate_snap_with_ratio(
            &screen,
            &config,
            win_pos,
            idx,
            &all_bubbles,
            edge_ratio,
        );
        if snap.should_dismiss {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                super::lifecycle::dismiss_bubble_from_drag(app_clone, idx).await;
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
    if let Some(s) = size
        && (!s.is_finite() || !(24.0..=256.0).contains(&s)) {
            return Err(format!("Invalid bubble size {}: expected finite value in 24..=256", s));
        }
    if let Some(r) = tuck_ratio
        && (!r.is_finite() || !(0.0..=1.0).contains(&r)) {
            return Err(format!("Invalid tuck_ratio {}: expected finite value in 0.0..=1.0", r));
        }
    if let Some(state) = app.try_state::<BubbleConfigState>()
        && let Ok(mut guard) = state.0.write() {
            if let Some(s) = size {
                guard.size = s;
            }
            if let Some(r) = tuck_ratio {
                guard.tuck_ratio = r;
            }
        }
    if let Some(s) = size {
        let size_str = classify_bubble_size(s).to_string();
        if let Some(chat_state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
            chat_state.update_user_config(&app, |c| {
                c.bubble_size = size_str;
            });
        }
    }
    let config = get_bubble_physics_config(&app);
    let screen = crate::movement::get_target_dashboard_screen(Some(&app));
    let bubble_size_px = (config.size * screen.scale_factor).round() as u32;
    for idx in 1..=crate::state::MAX_ACCOUNTS {
        let label = get_bubble_window_label(idx);
        if let Some(window) = app.get_webview_window(&label) {
            resize_bubble_window(&window, config.size);
        }
        if crate::operating_systems::current::bubble::has_native_bubble(idx) {
            crate::operating_systems::current::bubble::set_native_bubble_size(idx, bubble_size_px);
            if let Some((bx, by, _, _)) = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(idx) {
                crate::operating_systems::current::bubble::update_cached_bubble_bounds(
                    idx,
                    bx,
                    by,
                    bubble_size_px,
                    bubble_size_px,
                );
            }
            if let Some(chat_state) = app.try_state::<Arc<ChatState>>()
                && let Some(mut rect) = chat_state.get_bubble_rect(idx) {
                    rect.width = bubble_size_px as i32;
                    rect.height = bubble_size_px as i32;
                    chat_state.set_bubble_rect(idx, rect);
                }
        }
    }
    if !config.is_free_position {
        let mut all_rects = get_active_bubble_rects(&app, None);
        for idx in 1..=crate::state::MAX_ACCOUNTS {
            let cur_pos = if crate::operating_systems::current::bubble::has_native_bubble(idx) {
                crate::operating_systems::current::bubble::NativeBubbleManager::global()
                    .get_bounds(idx)
                    .map(|(x, y, _, _)| (x, y, true))
            } else {
                let label = get_bubble_window_label(idx);
                app.get_webview_window(&label)
                    .filter(|w| w.is_visible().unwrap_or(false))
                    .and_then(|w| w.outer_position().ok().map(|p| (p.x, p.y, false)))
            };
            if let Some((cur_x, cur_y, is_native)) = cur_pos {
                let stay_untucked = crate::presentation::bubble::is_bubble_stay_untucked(&app, idx);
                let edge_ratio = BubbleMovementEngine::effective_edge_ratio(&config, stay_untucked);
                let snap = BubbleMovementEngine::calculate_snap_with_ratio(
                    &screen,
                    &config,
                    PhysicalPosition { x: cur_x, y: cur_y },
                    idx,
                    &all_rects,
                    edge_ratio,
                );
                if snap.should_dismiss {
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        super::lifecycle::dismiss_bubble_from_drag(app_clone, idx).await;
                    });
                    continue;
                }
                if let Some(existing) = all_rects.iter_mut().find(|r| r.account_index == idx) {
                    existing.x = snap.target_x;
                    existing.y = snap.target_y;
                    existing.width = bubble_size_px as i32;
                    existing.height = bubble_size_px as i32;
                }
                if is_native {
                    crate::operating_systems::current::bubble::NativeBubbleManager::global().dispatch(
                        crate::operating_systems::current::bubble::NativeBubbleAction::AnimateTo {
                            account_index: idx,
                            target_x: snap.target_x,
                            target_y: snap.target_y,
                            duration_ms: 160,
                        },
                    );
                    crate::operating_systems::current::bubble::update_cached_bubble_bounds(
                        idx,
                        snap.target_x,
                        snap.target_y,
                        bubble_size_px,
                        bubble_size_px,
                    );
                } else if let Some(window) = app.get_webview_window(&get_bubble_window_label(idx)) {
                    let _ = window.set_position(Position::Physical(PhysicalPosition {
                        x: snap.target_x,
                        y: snap.target_y,
                    }));
                }
                if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
                    chat_state.set_bubble_rect(
                        idx,
                        BubbleRect {
                            account_index: idx,
                            x: snap.target_x,
                            y: snap.target_y,
                            width: bubble_size_px as i32,
                            height: bubble_size_px as i32,
                        },
                    );
                }
            }
        }
    }
    let size_str = classify_bubble_size(config.size);
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
    if !matches!(mode.as_str(), "auto_tuck" | "free") {
        return Err(format!("Invalid bubble_position_mode '{}': expected auto_tuck or free", mode));
    }
    let is_free = mode == "free";
    if let Some(state) = app.try_state::<BubbleConfigState>()
        && let Ok(mut guard) = state.0.write() {
            guard.is_free_position = is_free;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_bubble_size_unified_thresholds() {
        assert_eq!(classify_bubble_size(40.0), "sm");
        assert_eq!(classify_bubble_size(48.0), "sm");
        assert_eq!(classify_bubble_size(49.0), "md");
        assert_eq!(classify_bubble_size(56.0), "md");
        assert_eq!(classify_bubble_size(64.0), "lg");
        assert_eq!(classify_bubble_size(72.0), "lg");
    }
}
