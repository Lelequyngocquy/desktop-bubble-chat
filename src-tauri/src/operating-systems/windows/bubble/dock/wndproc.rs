use std::time::Instant;

use crate::operating_systems::windows::bubble::window::{
    DefWindowProcW, DeleteDC, DeleteObject, GetCursorPos, GetWindowLongPtrW, KillTimer,
    LoadCursorW, ReleaseCapture, SelectObject, SetCapture, SetCursor, SetWindowLongPtrW,
    ShowWindow, TrackMouseEvent, ValidateRect, GWLP_USERDATA, HTCLIENT, HTTRANSPARENT, IDC_HAND,
    MA_NOACTIVATE, POINT, SW_HIDE, TME_LEAVE, TRACKMOUSEEVENT, WM_CAPTURECHANGED, WM_DESTROY,
    WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSELEAVE, WM_MOUSEMOVE,
    WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WM_SETCURSOR, WM_TIMER,
};
use crate::presentation::bubble::dock::{
    execute_dock_action, hit_test_dock_screen_point, ActionDockLayout, DockActionId,
};
use super::state::{ActionDockWindowState, TIMER_ID_DOCK_ANIM};

/// Pure Win32 WM_NCHITTEST evaluator keeping inter-droplet gaps click-through
pub fn hit_test_action_dock_screen_point(
    screen_x: i32,
    screen_y: i32,
    layout: &ActionDockLayout,
    is_visible: bool,
    is_pressed: bool,
) -> isize {
    if !is_visible {
        return HTTRANSPARENT;
    }
    if is_pressed {
        return HTCLIENT;
    }
    if hit_test_dock_screen_point(screen_x, screen_y, layout).is_some() {
        HTCLIENT
    } else {
        HTTRANSPARENT
    }
}

/// Window Procedure for the non-activating singleton Win32 Liquid Glass ActionDock
///
/// # Safety
/// Caller must ensure `hwnd` is a valid Win32 window handle adhering to Windows API conventions.
pub unsafe extern "system" fn native_action_dock_wndproc(
    hwnd: *mut std::ffi::c_void,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ActionDockWindowState };
    match msg {
        WM_MOUSEACTIVATE => MA_NOACTIVATE,
        WM_NCHITTEST => {
            if let Some(state) = unsafe { state_ptr.as_ref() } {
                let screen_x = (lparam & 0xFFFF) as i16 as i32;
                let screen_y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                hit_test_action_dock_screen_point(
                    screen_x,
                    screen_y,
                    &state.layout,
                    state.is_visible,
                    state.animator.pressed_action.is_some(),
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
        WM_MOUSEMOVE => {
            let mut pt = POINT { x: 0, y: 0 };
            unsafe { GetCursorPos(&mut pt); }
            let mut arm_track = false;
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                if !state.is_tracking_mouse {
                    state.is_tracking_mouse = true;
                    arm_track = true;
                }
                let hit = hit_test_dock_screen_point(pt.x, pt.y, &state.layout);
                if state.animator.set_hovered(hit) {
                    state.render_layered_buffer();
                    state.sync_anim_timer();
                }
            }
            if arm_track {
                unsafe {
                    let mut tme = TRACKMOUSEEVENT {
                        cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dw_flags: TME_LEAVE,
                        hwnd_track: hwnd,
                        dw_hover_time: 0,
                    };
                    TrackMouseEvent(&mut tme);
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                state.is_tracking_mouse = false;
                let changed_h = state.animator.set_hovered(None);
                let changed_p = state.animator.set_pressed(None);
                if changed_h || changed_p {
                    state.render_layered_buffer();
                    state.sync_anim_timer();
                }
            }
            0
        }
        WM_LBUTTONDOWN => {
            let mut pt = POINT { x: 0, y: 0 };
            unsafe { GetCursorPos(&mut pt); }
            let mut should_capture = false;
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                let hit = hit_test_dock_screen_point(pt.x, pt.y, &state.layout);
                if hit.is_some() {
                    should_capture = true;
                    state.animator.set_hovered(hit);
                    if state.animator.set_pressed(hit) {
                        state.render_layered_buffer();
                        state.sync_anim_timer();
                    }
                }
            }
            if should_capture {
                unsafe { SetCapture(hwnd); }
            }
            0
        }
        WM_LBUTTONUP => {
            let mut pt = POINT { x: 0, y: 0 };
            unsafe { GetCursorPos(&mut pt); }
            let click_payload = {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    let pressed = state.animator.pressed_action;
                    let hit = hit_test_dock_screen_point(pt.x, pt.y, &state.layout);
                    let triggered = if pressed.is_some() && pressed == hit { pressed } else { None };
                    state.animator.set_pressed(None);
                    state.animator.set_hovered(hit);
                    state.render_layered_buffer();
                    state.sync_anim_timer();
                    triggered.and_then(|action| state.app_handle.clone().map(|app| (app, state.account_index, action)))
                } else {
                    None
                }
            };
            unsafe { ReleaseCapture(); }
            if let Some((app, account_index, action)) = click_payload {
                tauri::async_runtime::spawn(async move {
                    execute_dock_action(&app, account_index, action);
                    if action != DockActionId::HideBubble {
                        crate::operating_systems::windows::bubble::sync_native_action_dock_state(&app, account_index, None);
                    }
                });
            }
            0
        }
        WM_CAPTURECHANGED => {
            if let Some(state) = unsafe { state_ptr.as_mut() }
                && state.animator.set_pressed(None) {
                    state.render_layered_buffer();
                    state.sync_anim_timer();
                }
            0
        }
        WM_TIMER => {
            if wparam == TIMER_ID_DOCK_ANIM
                && let Some(state) = unsafe { state_ptr.as_mut() } {
                    let now = Instant::now();
                    let dt_ms = now.duration_since(state.last_tick).as_secs_f32() * 1000.0;
                    state.last_tick = now;
                    let finished_hide = state.animator.step(dt_ms);
                    if finished_hide {
                        state.is_visible = false;
                        state.is_tracking_mouse = false;
                        state.sync_hover_corridor_cache();
                        state.sync_anim_timer();
                        unsafe { ShowWindow(hwnd, SW_HIDE); }
                    } else {
                        state.render_layered_buffer();
                        state.sync_anim_timer();
                    }
                }
            0
        }
        WM_NCDESTROY => {
            unsafe {
                KillTimer(hwnd, TIMER_ID_DOCK_ANIM);
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ActionDockWindowState;
                if !ptr.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    let mut state = Box::from_raw(ptr);
                    state.is_visible = false;
                    state.sync_hover_corridor_cache();
                    if !state.hdc_mem.is_null() {
                        if !state.hbitmap.is_null() {
                            SelectObject(state.hdc_mem, state.old_bitmap);
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
    use crate::movement::ScreenMetrics;
    use crate::presentation::bubble::dock::compute_action_dock_layout;

    #[test]
    fn test_windows_action_dock_nchittest_click_through_and_negative_monitor() {
        let screen = ScreenMetrics { x: -1920, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let layout = compute_action_dock_layout(-1900, 120, 56, &screen, 12.0, 60.0);
        let pin = layout.slots[0];
        let resize = layout.slots[1];
        let pin_sx = layout.window_x + pin.local_cx.round() as i32;
        let pin_sy = layout.window_y + pin.local_cy.round() as i32;
        assert_eq!(hit_test_action_dock_screen_point(pin_sx, pin_sy, &layout, true, false), HTCLIENT);
        assert_eq!(hit_test_action_dock_screen_point(pin_sx, pin_sy, &layout, false, false), HTTRANSPARENT);
        let gap_sy = layout.window_y + ((pin.local_cy + resize.local_cy) * 0.5).round() as i32;
        assert_eq!(hit_test_action_dock_screen_point(pin_sx, gap_sy, &layout, true, false), HTTRANSPARENT);
        assert_eq!(hit_test_action_dock_screen_point(pin_sx, gap_sy, &layout, true, true), HTCLIENT);
    }
}
