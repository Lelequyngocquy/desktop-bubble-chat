use std::sync::Arc;
use tauri::{Manager, PhysicalPosition};

use super::super::ffi::*;
use super::super::state::BubbleWindowState;
use super::super::types::*;
use crate::movement::BubbleMovementEngine;
use crate::state::ChatState;

/// Handles 60 FPS spring interpolation timer ticks without holding &mut across SetWindowPos
pub(super) unsafe fn handle_spring_timer(hwnd: *mut std::ffi::c_void, state_ptr: *mut BubbleWindowState) {
    let tick_res = {
        if let Some(state) = unsafe { state_ptr.as_mut() } {
            if let Some((cur_x, cur_y, is_finished)) = state.anim_state.step() {
                state.x = cur_x;
                state.y = cur_y;
                crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                    state.account_index,
                    cur_x,
                    cur_y,
                    state.bubble_size,
                    state.bubble_size,
                );
                if is_finished {
                    state.update_layered_buffer();
                    if let Some(app) = &state.app_handle
                        && let Some(cs) = app.try_state::<Arc<ChatState>>() {
                            cs.set_bubble_rect(
                                state.account_index,
                                crate::movement::BubbleRect {
                                    account_index: state.account_index,
                                    x: cur_x,
                                    y: cur_y,
                                    width: state.bubble_size as i32,
                                    height: state.bubble_size as i32,
                                },
                            );
                        }
                }
                Some((Some((cur_x - BUBBLE_CANVAS_PAD_X, cur_y - BUBBLE_CANVAS_PAD_Y)), is_finished))
            } else {
                Some((None, true))
            }
        } else {
            None
        }
    };
    if let Some((pos_opt, stop_timer)) = tick_res {
        if let Some((win_x, win_y)) = pos_opt {
            unsafe {
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    win_x,
                    win_y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        if stop_timer {
            unsafe { KillTimer(hwnd, TIMER_ID_SPRING); }
        }
    }
}

/// Handles hover-leave debounce timer and tucks bubble if chat is closed and unpinned
pub(super) unsafe fn handle_hover_leave_timer(hwnd: *mut std::ffi::c_void, state_ptr: *mut BubbleWindowState) {
    unsafe { KillTimer(hwnd, TIMER_ID_HOVER_LEAVE); }
    if let Some(state) = unsafe { state_ptr.as_mut() }
        && !state.is_dragging && !state.is_hovered {
            let is_chat_open = if let Some(app) = &state.app_handle {
                crate::platforms::messenger::windows::get_messenger_webview(app, state.account_index)
                    .and_then(|w| w.is_visible().ok())
                    .unwrap_or(false)
            } else {
                false
            };
            let is_pinned = state.app_handle.as_ref()
                .and_then(|app| app.try_state::<std::sync::Arc<crate::state::ChatState>>())
                .map(|cs| cs.is_chat_pinned(state.account_index))
                .unwrap_or(false);
            if !is_chat_open && !is_pinned {
                let is_free = state.app_handle.as_ref()
                    .map(|app| crate::movement::get_bubble_physics_config(app).is_free_position)
                    .unwrap_or(false);
                if !is_free {
                    state.tuck();
                }
            }
        }
}

/// Finalizes bubble drag interaction, resolves collisions, and snaps to target edge without holding &mut across Win32 calls
pub(super) unsafe fn finish_drag_and_snap(hwnd: *mut std::ffi::c_void, state_ptr: *mut BubbleWindowState) {
    let plan = {
        let Some(state) = (unsafe { state_ptr.as_mut() }) else {
            return;
        };
        state.set_layered_alpha(255);
        let (target_x, target_y, should_dismiss, displaced_slots) = if let Some(app) = &state.app_handle {
            let screen = crate::movement::get_target_dashboard_screen(Some(app));
            let config = crate::movement::get_bubble_physics_config(app);
            if config.is_free_position {
                let scale = screen.scale_factor;
                let bubble_size = (config.size * scale) as i32;
                let dismiss_margin = (config.dismiss_margin * scale) as i32;
                let dismiss_threshold = screen.y + screen.height - dismiss_margin;
                let should_dismiss = state.y + bubble_size >= dismiss_threshold;
                if should_dismiss {
                    (state.x, state.y, true, Vec::new())
                } else {
                    let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
                    let (target_x, target_y) = crate::movement::CollisionResolver::resolve_2d_free_collision(
                        state.account_index,
                        state.x,
                        state.y,
                        &screen,
                        &config,
                        &all_bubbles,
                    );
                    if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                        cs.set_bubble_rect(
                            state.account_index,
                            crate::movement::BubbleRect {
                                account_index: state.account_index,
                                x: target_x,
                                y: target_y,
                                width: bubble_size,
                                height: bubble_size,
                            },
                        );
                    }
                    (target_x, target_y, false, Vec::new())
                }
            } else {
                let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
                let snap = BubbleMovementEngine::calculate_mutual_snap(
                    &screen,
                    &config,
                    PhysicalPosition { x: state.x, y: state.y },
                    state.account_index,
                    &all_bubbles,
                );
                if !snap.should_dismiss {
                    let bubble_w = (config.size * screen.scale_factor) as i32;
                    if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                        cs.set_bubble_rect(
                            state.account_index,
                            crate::movement::BubbleRect {
                                account_index: state.account_index,
                                x: snap.target_x,
                                y: snap.target_y,
                                width: bubble_w,
                                height: bubble_w,
                            },
                        );
                    }
                }
                (snap.target_x, snap.target_y, snap.should_dismiss, snap.column_slots)
            }
        } else {
            let screen = crate::movement::get_target_dashboard_screen(None);
            let mid_x = screen.x + screen.width / 2;
            let target_x = if state.x + (state.bubble_size as i32 / 2) < mid_x {
                screen.x
            } else {
                screen.x + screen.width - state.bubble_size as i32
            };
            (target_x, state.y, false, Vec::new())
        };
        if !should_dismiss {
            crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                state.account_index,
                target_x,
                target_y,
                state.bubble_size,
                state.bubble_size,
            );
            state.anim_state.start(state.x, state.y, target_x, target_y, 160);
        }
        (target_x, state.account_index, should_dismiss, displaced_slots, state.app_handle.clone())
    };
    let (target_x, account_index, should_dismiss, displaced_slots, app_opt) = plan;
    if should_dismiss {
        unsafe { ShowWindow(hwnd, SW_HIDE); }
        if let Some(app_clone) = app_opt {
            tauri::async_runtime::spawn(async move {
                crate::commands::bubble::dismiss_bubble_from_drag(app_clone, account_index).await;
            });
        }
    } else {
        unsafe { SetTimer(hwnd, TIMER_ID_SPRING, 16, None); }
        if let Some(app) = app_opt {
            let config = crate::movement::get_bubble_physics_config(&app);
            let screen = crate::movement::get_target_dashboard_screen(Some(&app));
            let bubble_w = (config.size * screen.scale_factor) as i32;
            for slot in displaced_slots {
                if slot.account_index != account_index {
                    crate::operating_systems::windows::bubble::NativeBubbleManager::global().dispatch(
                        crate::operating_systems::windows::bubble::NativeBubbleAction::AnimateTo {
                            account_index: slot.account_index,
                            target_x,
                            target_y: slot.target_y,
                            duration_ms: 160,
                        },
                    );
                    crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                        slot.account_index,
                        target_x,
                        slot.target_y,
                        bubble_w as u32,
                        bubble_w as u32,
                    );
                    if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                        cs.set_bubble_rect(
                            slot.account_index,
                            crate::movement::BubbleRect {
                                account_index: slot.account_index,
                                x: target_x,
                                y: slot.target_y,
                                width: bubble_w,
                                height: bubble_w,
                            },
                        );
                    }
                }
            }
        }
    }
}
