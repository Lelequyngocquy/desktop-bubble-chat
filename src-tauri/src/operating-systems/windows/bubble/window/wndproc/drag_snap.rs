use std::sync::Arc;
use tauri::Manager;

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

/// Handles hover-leave debounce timer and tucks bubble if chat is closed, unpinned, and unalerted
pub(super) unsafe fn handle_hover_leave_timer(hwnd: *mut std::ffi::c_void, state_ptr: *mut BubbleWindowState) {
    unsafe { KillTimer(hwnd, TIMER_ID_HOVER_LEAVE); }
    if let Some(state) = unsafe { state_ptr.as_mut() } {
        let flags = state.interaction_flags();
        if !flags.is_dragging && !flags.is_hovered && !flags.is_chat_open && !flags.is_chat_pinned {
            state.tuck();
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
        let drop_plan = if let Some(app) = &state.app_handle {
            let screen = crate::movement::get_target_dashboard_screen(Some(app));
            let config = crate::movement::get_bubble_physics_config(app);
            let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
            let stay_untucked = crate::presentation::bubble::is_bubble_stay_untucked(app, state.account_index);
            let computed = BubbleMovementEngine::compute_drop_plan(
                state.account_index,
                state.x,
                state.y,
                stay_untucked,
                &screen,
                &config,
                &all_bubbles,
            );
            if !computed.should_dismiss {
                let bubble_w = (config.size * screen.scale_factor) as i32;
                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                    cs.set_bubble_rect(
                        state.account_index,
                        crate::movement::BubbleRect {
                            account_index: state.account_index,
                            x: computed.target_x,
                            y: computed.target_y,
                            width: bubble_w,
                            height: bubble_w,
                        },
                    );
                }
            }
            computed
        } else {
            let screen = crate::movement::get_target_dashboard_screen(None);
            let mid_x = screen.x + screen.width / 2;
            let target_x = if state.x + (state.bubble_size as i32 / 2) < mid_x {
                screen.x
            } else {
                screen.x + screen.width - state.bubble_size as i32
            };
            crate::movement::BubbleDropPlan { target_x, target_y: state.y, should_dismiss: false, displaced_siblings: Vec::new() }
        };
        if !drop_plan.should_dismiss {
            crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                state.account_index,
                drop_plan.target_x,
                drop_plan.target_y,
                state.bubble_size,
                state.bubble_size,
            );
            state.anim_state.start(state.x, state.y, drop_plan.target_x, drop_plan.target_y, 160);
        }
        (state.account_index, drop_plan.should_dismiss, drop_plan.displaced_siblings, state.app_handle.clone())
    };
    let (account_index, should_dismiss, displaced_siblings, app_opt) = plan;
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
            for (sib_idx, sib_x, sib_y) in displaced_siblings {
                crate::operating_systems::windows::bubble::NativeBubbleManager::global().dispatch(
                    crate::operating_systems::windows::bubble::NativeBubbleAction::AnimateTo {
                        account_index: sib_idx,
                        target_x: sib_x,
                        target_y: sib_y,
                        duration_ms: 160,
                    },
                );
                crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                    sib_idx,
                    sib_x,
                    sib_y,
                    bubble_w as u32,
                    bubble_w as u32,
                );
                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                    cs.set_bubble_rect(
                        sib_idx,
                        crate::movement::BubbleRect {
                            account_index: sib_idx,
                            x: sib_x,
                            y: sib_y,
                            width: bubble_w,
                            height: bubble_w,
                        },
                    );
                }
            }
            let app_redock = app.clone();
            tauri::async_runtime::spawn(async move {
                crate::presentation::bubble::dock::on_bubble_drag_finished(&app_redock, account_index);
            });
        }
    }
}
