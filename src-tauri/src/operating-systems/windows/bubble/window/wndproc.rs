use std::sync::Arc;
use tauri::{Manager, PhysicalPosition};

use super::ffi::*;
use super::state::BubbleWindowState;
use super::types::*;
use crate::movement::BubbleMovementEngine;
use crate::state::ChatState;

/// Window Procedure handling Win32 messages for native bubbles.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid Win32 window handle and parameters adhere to Windows API conventions.
pub unsafe extern "system" fn native_bubble_wndproc(
    hwnd: *mut std::ffi::c_void,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut BubbleWindowState };
    match msg {
        WM_SETCURSOR => {
            unsafe {
                let h_cursor = LoadCursorW(std::ptr::null_mut(), IDC_HAND as *const u16);
                SetCursor(h_cursor);
            }
            1
        }
        WM_LBUTTONDOWN => {
            let down_info = {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    let is_on_top = state.app_handle.as_ref()
                        .and_then(|a| a.try_state::<Arc<ChatState>>())
                        .map(|s| s.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst))
                        .unwrap_or(false);
                    state.anim_state.stop();
                    state.is_dragging = true;
                    state.has_dragged = false;
                    state.drag_start_win = POINT { x: state.x, y: state.y };
                    Some((is_on_top, state.account_index, state.app_handle.clone()))
                } else {
                    None
                }
            };
            if let Some((is_on_top, account_index, app_opt)) = down_info {
                if is_on_top {
                    unsafe {
                        SetWindowPos(hwnd, -1_isize as *mut std::ffi::c_void, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                    }
                }
                let mut cursor_pt = POINT { x: 0, y: 0 };
                unsafe {
                    KillTimer(hwnd, TIMER_ID_SPRING);
                    KillTimer(hwnd, TIMER_ID_HOVER_LEAVE);
                    SetCapture(hwnd);
                    GetCursorPos(&mut cursor_pt);
                }
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    state.drag_start_cursor = cursor_pt;
                }
                crate::utils::trim_controller::record_user_interaction(account_index);
                if let Some(app) = app_opt.as_ref() {
                    crate::platforms::messenger::windows::set_chat_window_memory_target(app, account_index, false);
                    crate::utils::trim_controller::cancel_auto_trim(account_index);
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            let mut cur_pt = POINT { x: 0, y: 0 };
            unsafe { GetCursorPos(&mut cur_pt); }
            let mut move_pos: Option<(i32, i32)> = None;
            let mut hover_init: Option<(u32, Option<tauri::AppHandle>)> = None;
            {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    if state.is_dragging {
                        let dx = cur_pt.x - state.drag_start_cursor.x;
                        let dy = cur_pt.y - state.drag_start_cursor.y;
                        // Drag threshold > 4px (16 sq px)
                        if dx * dx + dy * dy >= 16 {
                            state.has_dragged = true;
                        }
                        if state.has_dragged {
                            let new_x = state.drag_start_win.x + dx;
                            let new_y = state.drag_start_win.y + dy;
                            state.x = new_x;
                            state.y = new_y;
                            crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                                state.account_index,
                                new_x,
                                new_y,
                                state.bubble_size,
                                state.bubble_size,
                            );
                            let screen = crate::movement::get_target_dashboard_screen(state.app_handle.as_ref());
                            let bottom_edge = screen.y + screen.height;
                            let dist = bottom_edge - new_y;
                            let start_fade_dist = (340.0 * screen.scale_factor) as i32;
                            let max_fade_dist = (140.0 * screen.scale_factor) as i32;
                            let alpha = if dist >= start_fade_dist {
                                255
                            } else if dist <= max_fade_dist {
                                64
                            } else {
                                let ratio = (start_fade_dist - dist) as f64 / (start_fade_dist - max_fade_dist) as f64;
                                ((1.0 - ratio * 0.75) * 255.0).clamp(64.0, 255.0) as u8
                            };
                            state.set_layered_alpha(alpha);
                            move_pos = Some((new_x - BUBBLE_CANVAS_PAD_X, new_y - BUBBLE_CANVAS_PAD_Y));
                        }
                    } else if !state.is_hovered {
                        state.is_hovered = true;
                        state.update_layered_buffer();
                        hover_init = Some((state.account_index, state.app_handle.clone()));
                    }
                }
            }
            if let Some((win_x, win_y)) = move_pos {
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
            } else if let Some((account_index, app_opt)) = hover_init {
                unsafe {
                    KillTimer(hwnd, TIMER_ID_HOVER_LEAVE);
                    let mut tme = TRACKMOUSEEVENT {
                        cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dw_flags: TME_LEAVE,
                        hwnd_track: hwnd,
                        dw_hover_time: 0,
                    };
                    TrackMouseEvent(&mut tme);
                }
                crate::utils::trim_controller::record_user_interaction(account_index);
                if let Some(app) = app_opt.as_ref() {
                    crate::platforms::messenger::windows::set_chat_window_memory_target(app, account_index, false);
                    crate::utils::trim_controller::cancel_auto_trim(account_index);
                    if !crate::movement::get_bubble_physics_config(app).is_free_position
                        && let Some(state) = unsafe { state_ptr.as_mut() } {
                            state.untuck();
                        }
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            let should_arm_timer = {
                if let Some(state) = unsafe { state_ptr.as_mut() }
                    && !state.is_dragging {
                        state.is_hovered = false;
                        state.update_layered_buffer();
                        true
                    } else {
                        false
                    }
            };
            if should_arm_timer {
                unsafe {
                    SetTimer(hwnd, TIMER_ID_HOVER_LEAVE, 80, None);
                }
            }
            0
        }
        WM_CAPTURECHANGED => {
            let capture_action = {
                if let Some(state) = unsafe { state_ptr.as_mut() }
                    && state.is_dragging {
                        state.is_dragging = false;
                        if state.has_dragged {
                            Some(true)
                        } else {
                            state.set_layered_alpha(255);
                            Some(false)
                        }
                    } else {
                        None
                    }
            };
            if capture_action == Some(true) {
                unsafe { finish_drag_and_snap(hwnd, state_ptr); }
            }
            0
        }
        WM_LBUTTONUP => {
            // Copy interaction state and end &mut borrow before ReleaseCapture synchronously sends WM_CAPTURECHANGED
            let release_action = {
                if let Some(state) = unsafe { state_ptr.as_mut() }
                    && state.is_dragging {
                        state.is_dragging = false;
                        Some((state.has_dragged, state.account_index, state.app_handle.clone()))
                    } else {
                        None
                    }
            };
            if let Some((has_dragged, idx, app_opt)) = release_action {
                unsafe { ReleaseCapture(); }
                if has_dragged {
                    unsafe { finish_drag_and_snap(hwnd, state_ptr); }
                } else if let Some(app_clone) = app_opt {
                    tauri::async_runtime::spawn(async move {
                        if let Some(chat_state) = app_clone.try_state::<Arc<ChatState>>() {
                            let _ = crate::platforms::messenger::services::ChatService::toggle_chat(
                                &app_clone,
                                &chat_state,
                                idx,
                            );
                        }
                    });
                }
            }
            0
        }
        WM_TIMER => {
            if wparam == TIMER_ID_SPRING {
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
            } else if wparam == TIMER_ID_HOVER_LEAVE {
                unsafe { KillTimer(hwnd, TIMER_ID_HOVER_LEAVE); }
                if let Some(state) = unsafe { state_ptr.as_mut() }
                    && !state.is_dragging && !state.is_hovered {
                        let is_chat_open = if let Some(app) = &state.app_handle {
                            let label = crate::platforms::messenger::windows::get_messenger_window_label(state.account_index);
                            app.get_webview_window(&label)
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
            0
        }
        WM_NCDESTROY => {
            unsafe {
                KillTimer(hwnd, TIMER_ID_SPRING);
                KillTimer(hwnd, TIMER_ID_HOVER_LEAVE);
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
                if !ptr.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    let mut state = Box::from_raw(ptr);
                    if !state.hdc_mem.is_null() {
                        SelectObject(state.hdc_mem, state.old_bitmap);
                        DeleteObject(state.hbitmap);
                        DeleteDC(state.hdc_mem);
                        state.hdc_mem = std::ptr::null_mut();
                    }
                }
            }
            0
        }
        WM_DESTROY => 0,
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            unsafe { ValidateRect(hwnd, std::ptr::null()); }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Finalizes bubble drag interaction, resolves collisions, and snaps to target edge without holding &mut across Win32 calls.
unsafe fn finish_drag_and_snap(hwnd: *mut std::ffi::c_void, state_ptr: *mut BubbleWindowState) {
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
                let _ = crate::commands::bubble::toggle_bubble(app_clone, false, Some(account_index)).await;
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
