use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

use crate::movement::{BubbleMovementEngine, BubbleRect, ScreenMetrics};
use super::webview::{get_bubble_window_label, get_or_create_bubble_window, resize_bubble_window};

/// Returns active bubble bounds as BubbleRect list
pub fn get_active_bubble_rects(
    app: &AppHandle,
    exclude_label: Option<&str>,
) -> Vec<BubbleRect> {
    let mut rects = Vec::new();
    let state_rects: std::collections::HashMap<u32, BubbleRect> = app
        .try_state::<Arc<crate::state::ChatState>>()
        .and_then(|s| s.active_bubble_rects.lock().ok().map(|g| g.clone()))
        .unwrap_or_default();
    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(exclude) = exclude_label {
            if label == exclude {
                continue;
            }
        }
        #[cfg(target_os = "windows")]
        {
            if crate::windows::native_bubble::NativeBubbleManager::global().has_bubble(idx) {
                if let Some((x, y, w, h)) = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(idx) {
                    rects.push(BubbleRect {
                        account_index: idx,
                        x,
                        y,
                        width: w as i32,
                        height: h as i32,
                    });
                    continue;
                }
            }
        }
        let win = app.get_webview_window(&label);
        if let Some(w) = win {
            let is_vis = w.is_visible().unwrap_or(false);
            if is_vis || state_rects.contains_key(&idx) {
                if let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) {
                    if pos.x != 0 || pos.y != 0 {
                        rects.push(BubbleRect {
                            account_index: idx,
                            x: pos.x,
                            y: pos.y,
                            width: size.width as i32,
                            height: size.height as i32,
                        });
                        continue;
                    }
                }
                if let Some(sr) = state_rects.get(&idx) {
                    rects.push(*sr);
                }
            }
        } else if let Some(sr) = state_rects.get(&idx) {
            rects.push(*sr);
        }
    }
    rects
}

/// Positions and displays bubble for account with collision avoidance and optional extra obstacles
pub fn ensure_bubble_positioned_and_visible_with_extra(
    app: &AppHandle,
    account_index: u32,
    extra_obstacles: &[BubbleRect],
) -> Result<BubbleRect, String> {
    #[cfg(target_os = "windows")]
    {
        let config = crate::movement::get_bubble_physics_config(app);
        let monitor = app
            .get_webview_window("main")
            .and_then(|m| {
                if m.is_visible().unwrap_or(false) {
                    m.current_monitor().ok().flatten()
                } else {
                    None
                }
            })
            .or_else(|| app.primary_monitor().ok().flatten());
        if let Some(mon) = monitor {
            let screen = ScreenMetrics::from_monitor(&mon);
            let bubble_w = (config.size * screen.scale_factor) as i32;
            if crate::windows::native_bubble::NativeBubbleManager::global().has_bubble(account_index) {
                if let Some((x, y, w, _h)) = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(account_index) {
                    if w != bubble_w as u32 {
                        crate::windows::native_bubble::set_native_bubble_size(account_index, bubble_w as u32);
                    }
                    let rect = BubbleRect {
                        account_index,
                        x,
                        y,
                        width: bubble_w,
                        height: bubble_w,
                    };
                    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                        chat_state.set_bubble_rect(account_index, rect);
                    }
                    return Ok(rect);
                }
            }
            let mut other_bubbles = get_active_bubble_rects(app, None);
            for obs in extra_obstacles {
                if !other_bubbles.iter().any(|b| b.account_index == obs.account_index) {
                    other_bubbles.push(*obs);
                }
            }
            let target_pos = BubbleMovementEngine::calculate_stacked_pos(&screen, &config, &other_bubbles);
            crate::windows::native_bubble::NativeBubbleManager::global().dispatch(
                crate::windows::native_bubble::NativeBubbleAction::Show {
                    account_index,
                    x: target_pos.x,
                    y: target_pos.y,
                    size: bubble_w as u32,
                    app: app.clone(),
                }
            );
            let rect = BubbleRect {
                account_index,
                x: target_pos.x,
                y: target_pos.y,
                width: bubble_w,
                height: bubble_w,
            };
            if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                chat_state.set_bubble_rect(account_index, rect);
            }
            let _ = app.emit("bubble-visibility-changed", serde_json::json!({
                "accountIndex": account_index,
                "visible": true,
                "x": target_pos.x,
                "y": target_pos.y
            }));
            if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                crate::platforms::messenger::windows::preload_messenger_window(
                    app,
                    account_index,
                    Arc::clone(&chat_state),
                );
            }
            return Ok(rect);
        }
    }
    let bubble_window = get_or_create_bubble_window(app, account_index)?;
    let win_label = bubble_window.label().to_string();
    let config = crate::movement::get_bubble_physics_config(app);
    resize_bubble_window(&bubble_window, config.size);
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
    let mut committed_pos = PhysicalPosition { x: 0, y: 0 };
    let mut bubble_w = 56;
    if let Some(monitor) = monitor {
        let win_pos = bubble_window.outer_position().unwrap_or_default();
        let screen = ScreenMetrics::from_monitor(&monitor);
        bubble_w = (config.size * screen.scale_factor) as i32;
        let is_visible = bubble_window.is_visible().unwrap_or(false);
        let is_out_of_bounds = win_pos.x < screen.x - bubble_w * 2
            || win_pos.x > screen.x + screen.width + bubble_w
            || (!is_visible && win_pos.x == 0 && win_pos.y == 0);
        if !is_visible || is_out_of_bounds {
            let mut other_bubbles = get_active_bubble_rects(app, Some(&win_label));
            for obs in extra_obstacles {
                if !other_bubbles.iter().any(|b| b.account_index == obs.account_index) {
                    other_bubbles.push(*obs);
                }
            }
            let target_pos = BubbleMovementEngine::calculate_stacked_pos(&screen, &config, &other_bubbles);
            let _ = bubble_window.set_position(tauri::Position::Physical(target_pos));
            committed_pos = target_pos;
        } else {
            committed_pos = win_pos;
        }
    }
    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
        let is_on_top = chat_state.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst);
        let _ = bubble_window.set_always_on_top(is_on_top);
    }
    let _ = bubble_window.show();
    let _ = bubble_window.unminimize();
    let rect = BubbleRect {
        account_index,
        x: committed_pos.x,
        y: committed_pos.y,
        width: bubble_w,
        height: bubble_w,
    };
    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
        chat_state.set_bubble_rect(account_index, rect);
    }
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": account_index,
        "visible": true,
        "x": committed_pos.x,
        "y": committed_pos.y
    }));
    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
        crate::platforms::messenger::windows::preload_messenger_window(
            app,
            account_index,
            Arc::clone(&chat_state),
        );
    }
    Ok(BubbleRect {
        account_index,
        x: committed_pos.x,
        y: committed_pos.y,
        width: bubble_w,
        height: bubble_w,
    })
}

/// Positions and displays bubble for account with collision avoidance
pub fn ensure_bubble_positioned_and_visible(app: &AppHandle, account_index: u32) -> Result<BubbleRect, String> {
    ensure_bubble_positioned_and_visible_with_extra(app, account_index, &[])
}

/// Tucks floating bubbles into screen edge
pub fn tuck_bubble(app: &AppHandle, account_index: Option<u32>) {
    let indices: Vec<u32> = match account_index {
        Some(idx) => vec![idx],
        None => (1..=5).collect(),
    };
    let config = crate::movement::get_bubble_physics_config(app);
    for idx in indices {
        crate::utils::trim_controller::schedule_auto_trim_on_tuck(idx, app.clone());
        let label = get_bubble_window_label(idx);
        let win = app.get_webview_window(&label);
        if let Some(bubble_window) = win {
            if let Ok(Some(monitor)) = bubble_window.current_monitor() {
                let screen = ScreenMetrics::from_monitor(&monitor);
                let win_pos = bubble_window.outer_position().unwrap_or_default();
                let tucked_pos = BubbleMovementEngine::calculate_hover_pos(
                    &screen,
                    &config,
                    win_pos.x,
                    win_pos.y,
                    false,
                );
                let _ = bubble_window.set_position(tauri::Position::Physical(tucked_pos));
            }
        } else {
            crate::windows::native_bubble::tuck_native_bubble(app, idx);
        }
    }
}
