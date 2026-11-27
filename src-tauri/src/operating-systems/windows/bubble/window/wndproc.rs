mod drag_snap;

use std::sync::Arc;
use tauri::Manager;

use super::ffi::*;
use super::state::BubbleWindowState;
use super::types::*;
use crate::state::ChatState;
use drag_snap::{finish_drag_and_snap, handle_hover_leave_timer, handle_spring_timer};

/// Evaluates Win32 WM_NCHITTEST result so transparent padding and outer ripple pixels stay click-through.
#[allow(clippy::too_many_arguments)]
pub fn hit_test_bubble_screen_point(
    screen_x: i32,
    screen_y: i32,
    bubble_x: i32,
    bubble_y: i32,
    canvas_w: u32,
    canvas_h: u32,
    bubble_size: u32,
    unread_count: u32,
    is_left_edge: bool,
    is_dragging: bool,
) -> isize {
    if is_dragging {
        return HTCLIENT;
    }
    let local_x = (screen_x - (bubble_x - BUBBLE_CANVAS_PAD_X)) as f32;
    let local_y = (screen_y - (bubble_y - BUBBLE_CANVAS_PAD_Y)) as f32;
    if crate::presentation::bubble::renderer::is_point_in_bubble_or_badge_top_down(
        local_x,
        local_y,
        canvas_w as f32,
        canvas_h as f32,
        bubble_size as f32,
        unread_count,
        is_left_edge,
    ) {
        HTCLIENT
    } else {
        HTTRANSPARENT
    }
}

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
        WM_NCHITTEST => {
            if let Some(state) = unsafe { state_ptr.as_ref() } {
                let screen_x = (lparam & 0xFFFF) as i16 as i32;
                let screen_y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                hit_test_bubble_screen_point(
                    screen_x,
                    screen_y,
                    state.x,
                    state.y,
                    state.width,
                    state.height,
                    state.bubble_size,
                    state.unread_count,
                    state.is_at_left_edge(),
                    state.is_dragging,
                )
            } else {
                HTTRANSPARENT
            }
        }
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
                    state.sync_heartbeat_pause();
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
            let mut drag_started_info: Option<(u32, tauri::AppHandle)> = None;
            let mut hover_init: Option<(u32, Option<tauri::AppHandle>)> = None;
            {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    if state.is_dragging {
                        let dx = cur_pt.x - state.drag_start_cursor.x;
                        let dy = cur_pt.y - state.drag_start_cursor.y;
                        let was_dragged = state.has_dragged;
                        // Drag threshold > 4px (16 sq px)
                        if dx * dx + dy * dy >= 16 {
                            state.has_dragged = true;
                        }
                        if !was_dragged && state.has_dragged
                            && let Some(app) = state.app_handle.clone() {
                                drag_started_info = Some((state.account_index, app));
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
                        state.sync_heartbeat_pause_flag();
                        state.sync_heartbeat_timer_and_render();
                        hover_init = Some((state.account_index, state.app_handle.clone()));
                    }
                }
            }
            if let Some((drag_idx, app)) = drag_started_info {
                crate::presentation::bubble::dock::on_bubble_drag_started(&app, drag_idx);
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
                        state.sync_heartbeat_pause_flag();
                        state.sync_heartbeat_timer_and_render();
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
                        state.sync_heartbeat_pause();
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
                        state.sync_heartbeat_pause();
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
            if wparam == TIMER_ID_PULSE {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    state.sync_heartbeat_timer_and_render();
                }
            } else if wparam == TIMER_ID_SPRING {
                unsafe { handle_spring_timer(hwnd, state_ptr); }
            } else if wparam == TIMER_ID_HOVER_LEAVE {
                unsafe { handle_hover_leave_timer(hwnd, state_ptr); }
            }
            0
        }
        WM_NCDESTROY => {
            unsafe {
                KillTimer(hwnd, TIMER_ID_PULSE);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_nchittest_click_through_on_outer_ripple_and_negative_monitor() {
        let bubble_x = -200;
        let bubble_y = 100;
        let bubble_size = 56;
        let canvas_w = bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        // Center of bubble hits HTCLIENT even on negative X monitor
        assert_eq!(hit_test_bubble_screen_point(-172, 128, bubble_x, bubble_y, canvas_w, canvas_h, bubble_size, 3, true, false), HTCLIENT);
        // Point 36px above center sits inside expanded canvas ripple halo -> must be HTTRANSPARENT
        assert_eq!(hit_test_bubble_screen_point(-172, 92, bubble_x, bubble_y, canvas_w, canvas_h, bubble_size, 3, true, false), HTTRANSPARENT);
        // Unread badge to the right of left-edge bubble hits HTCLIENT when unread > 0
        assert_eq!(hit_test_bubble_screen_point(-138, 120, bubble_x, bubble_y, canvas_w, canvas_h, bubble_size, 3, true, false), HTCLIENT);
        assert_eq!(hit_test_bubble_screen_point(-138, 120, bubble_x, bubble_y, canvas_w, canvas_h, bubble_size, 0, true, false), HTTRANSPARENT);
        // Active drag keeps HTCLIENT everywhere
        assert_eq!(hit_test_bubble_screen_point(-172, 92, bubble_x, bubble_y, canvas_w, canvas_h, bubble_size, 3, true, true), HTCLIENT);
    }
}
