use std::sync::Arc;
use tauri::{Manager, PhysicalPosition};

use super::ffi::*;
use super::state::BubbleWindowState;
use super::types::*;
use crate::movement::BubbleMovementEngine;
use crate::state::ChatState;

/// Window Procedure handling Win32 messages for native bubbles.
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
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                unsafe {
                    KillTimer(hwnd, TIMER_ID_SPRING);
                    KillTimer(hwnd, TIMER_ID_HOVER_LEAVE);
                    SetCapture(hwnd);
                    GetCursorPos(&mut state.drag_start_cursor);
                }
                state.anim_state.stop();
                state.is_dragging = true;
                state.has_dragged = false;
                state.drag_start_win = POINT { x: state.x, y: state.y };
                crate::utils::trim_controller::record_user_interaction(state.account_index);
                if let Some(app) = state.app_handle.as_ref() {
                    crate::platforms::messenger::windows::set_chat_window_memory_target(app, state.account_index, false);
                    crate::utils::trim_controller::cancel_auto_trim(state.account_index);
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                if state.is_dragging {
                    let mut cur_pt = POINT { x: 0, y: 0 };
                    unsafe { GetCursorPos(&mut cur_pt); }
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
                        // Bottom dismiss fade opacity
                        let screen = crate::movement::get_target_dashboard_screen(state.app_handle.as_ref());
                        let bottom_edge = screen.y + screen.height;
                        let dist = bottom_edge - new_y;
                        let start_fade_dist = (340.0 * screen.scale_factor) as i32;
                        let max_fade_dist = (140.0 * screen.scale_factor) as i32;
                        let alpha = if dist >= start_fade_dist {
                            255
                        } else if dist <= max_fade_dist {
                            64 // 0.25 opacity
                        } else {
                            let ratio = (start_fade_dist - dist) as f64 / (start_fade_dist - max_fade_dist) as f64;
                            ((1.0 - ratio * 0.75) * 255.0).clamp(64.0, 255.0) as u8
                        };
                        state.set_layered_alpha(alpha);
                        unsafe {
                            SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                new_x - BUBBLE_CANVAS_PAD_X,
                                new_y - BUBBLE_CANVAS_PAD_Y,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                        }
                    }
                } else if !state.is_hovered {
                    // Mouse enter: pre-warm webview and untuck
                    state.is_hovered = true;
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
                    state.update_layered_buffer();
                    crate::utils::trim_controller::record_user_interaction(state.account_index);
                    if let Some(app) = state.app_handle.as_ref() {
                        crate::platforms::messenger::windows::set_chat_window_memory_target(app, state.account_index, false);
                        crate::utils::trim_controller::cancel_auto_trim(state.account_index);
                    }
                    let is_free = state.app_handle.as_ref()
                        .map(|app| crate::movement::get_bubble_physics_config(app).is_free_position)
                        .unwrap_or(false);
                    if !is_free {
                        state.untuck();
                    }
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                if !state.is_dragging {
                    state.is_hovered = false;
                    state.update_layered_buffer();
                    unsafe {
                        // 80ms debounce for mouse leave jitter
                        SetTimer(hwnd, TIMER_ID_HOVER_LEAVE, 80, None);
                    }
                }
            }
            0
        }
        WM_CAPTURECHANGED => {
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                if state.is_dragging {
                    state.is_dragging = false;
                    state.set_layered_alpha(255);
                }
            }
            0
        }
        WM_LBUTTONUP => {
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                if state.is_dragging {
                    state.is_dragging = false;
                    // Release capture after clearing is_dragging so WM_CAPTURECHANGED
                    // fires with is_dragging already false and becomes a no-op.
                    unsafe { ReleaseCapture(); }
                    if state.has_dragged {
                        // Restore full opacity
                        state.set_layered_alpha(255);
                        // Drag finished -> trigger snap to edge
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
                        if should_dismiss {
                            unsafe { ShowWindow(hwnd, SW_HIDE); }
                            if let Some(app) = &state.app_handle {
                                let app_clone = app.clone();
                                let idx = state.account_index;
                                tauri::async_runtime::spawn(async move {
                                    let _ = crate::commands::bubble::toggle_bubble(app_clone, false, Some(idx)).await;
                                });
                            }
                        } else {
                            crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                                state.account_index,
                                target_x,
                                target_y,
                                state.bubble_size,
                                state.bubble_size,
                            );
                            state.anim_state.start(state.x, state.y, target_x, target_y, 160);
                            unsafe {
                                SetTimer(hwnd, TIMER_ID_SPRING, 16, None);
                            }
                            if let Some(app) = &state.app_handle {
                                let config = crate::movement::get_bubble_physics_config(app);
                                let screen = crate::movement::get_target_dashboard_screen(Some(app));
                                let bubble_w = (config.size * screen.scale_factor) as i32;
                                for slot in displaced_slots {
                                    if slot.account_index != state.account_index {
                                        crate::operating_systems::windows::bubble::NativeBubbleManager::global().dispatch(
                                            crate::operating_systems::windows::bubble::NativeBubbleAction::AnimateTo {
                                                account_index: slot.account_index,
                                                target_x,
                                                target_y: slot.target_y,
                                                duration_ms: 160,
                                            }
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
                    } else {
                        // Click action (toggle chat)
                        if let Some(app) = &state.app_handle {
                            let app_clone = app.clone();
                            let idx = state.account_index;
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
                }
            }
            0
        }
        WM_TIMER => {
            if wparam == TIMER_ID_SPRING {
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
                        unsafe {
                            SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                cur_x - BUBBLE_CANVAS_PAD_X,
                                cur_y - BUBBLE_CANVAS_PAD_Y,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                        }
                        if is_finished {
                            unsafe { KillTimer(hwnd, TIMER_ID_SPRING); }
                            state.update_layered_buffer();
                            if let Some(app) = &state.app_handle {
                                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
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
                        }
                    } else {
                        unsafe { KillTimer(hwnd, TIMER_ID_SPRING); }
                    }
                }
            } else if wparam == TIMER_ID_HOVER_LEAVE {
                unsafe { KillTimer(hwnd, TIMER_ID_HOVER_LEAVE); }
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    if !state.is_dragging && !state.is_hovered {
                        let is_chat_open = if let Some(app) = &state.app_handle {
                            let label = crate::platforms::messenger::windows::get_messenger_window_label(state.account_index);
                            app.get_webview_window(&label)
                                .and_then(|w| w.is_visible().ok())
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        if !is_chat_open {
                            let is_free = state.app_handle.as_ref()
                                .map(|app| crate::movement::get_bubble_physics_config(app).is_free_position)
                                .unwrap_or(false);
                            if !is_free {
                                state.tuck();
                            }
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
        WM_PAINT => 0,
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
