mod drag_snap;

use std::sync::Arc;
use tauri::Manager;

use super::ffi::*;
use super::state::BubbleWindowState;
use super::types::*;
use crate::state::ChatState;
use drag_snap::{finish_drag_and_snap, handle_hover_leave_timer, handle_spring_timer};

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
                        state.is_hovered = false;
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
                        if state.has_dragged {
                            state.is_hovered = false;
                        }
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
                unsafe { handle_spring_timer(hwnd, state_ptr); }
            } else if wparam == TIMER_ID_HOVER_LEAVE {
                unsafe { handle_hover_leave_timer(hwnd, state_ptr); }
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
                        if !state.hbitmap.is_null() {
                            DeleteObject(state.hbitmap);
                            state.hbitmap = std::ptr::null_mut();
                        }
                        DeleteDC(state.hdc_mem);
                        state.hdc_mem = std::ptr::null_mut();
                    } else if !state.hbitmap.is_null() {
                        DeleteObject(state.hbitmap);
                        state.hbitmap = std::ptr::null_mut();
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
