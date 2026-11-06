use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::movement::{BubbleMovementEngine, BubbleRect};
use super::webview::get_bubble_window_label;
#[cfg(not(target_os = "windows"))]
use super::webview::{get_or_create_bubble_window, resize_bubble_window};

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
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let bubble_w = (config.size * screen.scale_factor) as i32;
        let mut other_bubbles = get_active_bubble_rects(app, None);
            for obs in extra_obstacles {
                if !other_bubbles.iter().any(|b| b.account_index == obs.account_index) {
                    other_bubbles.push(*obs);
                }
            }
            if crate::windows::native_bubble::NativeBubbleManager::global().has_bubble(account_index) {
                if let Some((x, y, w, _h)) = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(account_index) {
                    if w != bubble_w as u32 {
                        crate::windows::native_bubble::set_native_bubble_size(account_index, bubble_w as u32);
                    }
                    if config.is_free_position {
                        let (target_x, target_y) = crate::movement::CollisionResolver::resolve_2d_free_collision(
                            account_index,
                            x,
                            y,
                            &screen,
                            &config,
                            &other_bubbles,
                        );
                        if target_x != x || target_y != y {
                            crate::windows::native_bubble::NativeBubbleManager::global().dispatch(
                                crate::windows::native_bubble::NativeBubbleAction::SetPosition {
                                    account_index,
                                    x: target_x,
                                    y: target_y,
                                }
                            );
                            crate::windows::native_bubble::manager::update_cached_bubble_bounds(
                                account_index,
                                target_x,
                                target_y,
                                bubble_w as u32,
                                bubble_w as u32,
                            );
                        }
                        let rect = BubbleRect {
                            account_index,
                            x: target_x,
                            y: target_y,
                            width: bubble_w,
                            height: bubble_w,
                        };
                        if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                            chat_state.set_bubble_rect(account_index, rect);
                        }
                        return Ok(rect);
                    }
                    let mid_x = screen.x + screen.width / 2;
                    let is_left_edge = x + (bubble_w / 2) < mid_x;
                    let layout = crate::movement::CollisionResolver::resolve_mutual_column_layout(
                        account_index,
                        y,
                        is_left_edge,
                        &screen,
                        &config,
                        &other_bubbles,
                    );
                    let target_y = layout
                        .iter()
                        .find(|s| s.account_index == account_index)
                        .map(|s| s.target_y)
                        .unwrap_or(y);
                    if target_y != y {
                        crate::windows::native_bubble::NativeBubbleManager::global().dispatch(
                            crate::windows::native_bubble::NativeBubbleAction::SetPosition {
                                account_index,
                                x,
                                y: target_y,
                            }
                        );
                    }
                    for slot in layout {
                        if slot.account_index != account_index {
                            crate::windows::native_bubble::NativeBubbleManager::global().dispatch(
                                crate::windows::native_bubble::NativeBubbleAction::AnimateTo {
                                    account_index: slot.account_index,
                                    target_x: x,
                                    target_y: slot.target_y,
                                    duration_ms: 160,
                                }
                            );
                            crate::windows::native_bubble::manager::update_cached_bubble_bounds(
                                slot.account_index,
                                x,
                                slot.target_y,
                                bubble_w as u32,
                                bubble_w as u32,
                            );
                            if let Some(cs) = app.try_state::<Arc<crate::state::ChatState>>() {
                                cs.set_bubble_rect(
                                    slot.account_index,
                                    BubbleRect {
                                        account_index: slot.account_index,
                                        x,
                                        y: slot.target_y,
                                        width: bubble_w,
                                        height: bubble_w,
                                    },
                                );
                            }
                        }
                    }
                    let rect = BubbleRect {
                        account_index,
                        x,
                        y: target_y,
                        width: bubble_w,
                        height: bubble_w,
                    };
                    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                        chat_state.set_bubble_rect(account_index, rect);
                    }
                    return Ok(rect);
                }
            }
            let target_pos = if config.is_free_position {
                let margin = (config.margin * screen.scale_factor) as i32;
                let default_x = screen.x + margin + (account_index.saturating_sub(1) as i32) * (bubble_w + (config.gap * screen.scale_factor) as i32);
                let default_y = screen.y + margin + 100;
                let (resolved_x, resolved_y) = crate::movement::CollisionResolver::resolve_2d_free_collision(
                    account_index,
                    default_x,
                    default_y,
                    &screen,
                    &config,
                    &other_bubbles,
                );
                tauri::PhysicalPosition { x: resolved_x, y: resolved_y }
            } else {
                BubbleMovementEngine::calculate_stacked_pos(&screen, &config, &other_bubbles)
            };
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
    #[cfg(not(target_os = "windows"))]
    {
        let bubble_window = get_or_create_bubble_window(app, account_index)?;
        let win_label = bubble_window.label().to_string();
        let config = crate::movement::get_bubble_physics_config(app);
        resize_bubble_window(&bubble_window, config.size);
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let win_pos = bubble_window.outer_position().unwrap_or_default();
        let bubble_w = (config.size * screen.scale_factor) as i32;
        let is_visible = bubble_window.is_visible().unwrap_or(false);
        let is_out_of_bounds = win_pos.x < screen.x - bubble_w * 2
            || win_pos.x > screen.x + screen.width + bubble_w
            || (!is_visible && win_pos.x == 0 && win_pos.y == 0);
        let committed_pos = if !is_visible || is_out_of_bounds {
            let mut other_bubbles = get_active_bubble_rects(app, Some(&win_label));
            for obs in extra_obstacles {
                if !other_bubbles.iter().any(|b| b.account_index == obs.account_index) {
                    other_bubbles.push(*obs);
                }
            }
            let target_pos = if config.is_free_position {
                let margin = (config.margin * screen.scale_factor) as i32;
                let default_x = screen.x + margin + (account_index.saturating_sub(1) as i32) * (bubble_w + (config.gap * screen.scale_factor) as i32);
                let default_y = screen.y + margin + 100;
                let (resolved_x, resolved_y) = crate::movement::CollisionResolver::resolve_2d_free_collision(
                    account_index,
                    default_x,
                    default_y,
                    &screen,
                    &config,
                    &other_bubbles,
                );
                tauri::PhysicalPosition { x: resolved_x, y: resolved_y }
            } else {
                BubbleMovementEngine::calculate_stacked_pos(&screen, &config, &other_bubbles)
            };
            let _ = bubble_window.set_position(tauri::Position::Physical(target_pos));
            target_pos
        } else {
            win_pos
        };
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
}

/// Positions and displays bubble for account with collision avoidance
pub fn ensure_bubble_positioned_and_visible(app: &AppHandle, account_index: u32) -> Result<BubbleRect, String> {
    ensure_bubble_positioned_and_visible_with_extra(app, account_index, &[])
}

/// Tucks floating bubbles into screen edge
pub fn tuck_bubble(app: &AppHandle, account_index: Option<u32>) {
    let config = crate::movement::get_bubble_physics_config(app);
    if config.is_free_position {
        return;
    }
    let indices: Vec<u32> = match account_index {
        Some(idx) => vec![idx],
        None => (1..=5).collect(),
    };
    for idx in indices {
        crate::utils::trim_controller::schedule_auto_trim_on_tuck(idx, app.clone());
        let label = get_bubble_window_label(idx);
        let win = app.get_webview_window(&label);
        if let Some(bubble_window) = win {
            let screen = crate::movement::get_target_dashboard_screen(Some(app));
            let win_pos = bubble_window.outer_position().unwrap_or_default();
            let tucked_pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                win_pos.x,
                win_pos.y,
                false,
            );
            let _ = bubble_window.set_position(tauri::Position::Physical(tucked_pos));
        } else {
            crate::windows::native_bubble::tuck_native_bubble(app, idx);
        }
    }
}
