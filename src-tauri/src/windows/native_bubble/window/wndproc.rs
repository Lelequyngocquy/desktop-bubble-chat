use std::sync::Arc;
use tauri::{Manager, PhysicalPosition};

use super::ffi::*;
use super::state::BubbleWindowState;
use super::types::*;
use crate::movement::{BubbleMovementEngine, ScreenMetrics};
use crate::state::ChatState;

/// Window Procedure xử lý toàn bộ thông điệp Win32 của Native Bubble
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

                    // Ngưỡng phát hiện kéo thả > 4px (16px bình phương) chuẩn mốc usePointerTracker
                    if dx * dx + dy * dy >= 16 {
                        state.has_dragged = true;
                    }

                    if state.has_dragged {
                        let new_x = state.drag_start_win.x + dx;
                        let new_y = state.drag_start_win.y + dy;
                        state.x = new_x;
                        state.y = new_y;

                        crate::windows::native_bubble::manager::update_cached_bubble_bounds(
                            state.account_index,
                            new_x,
                            new_y,
                            state.width,
                            state.height,
                        );

                        // Hiệu ứng làm mờ khi kéo sát đáy màn hình (Bottom Dismiss Opacity)
                        if let Some(app) = &state.app_handle {
                            let monitor = app
                                .get_webview_window("main")
                                .and_then(|m| m.current_monitor().ok().flatten())
                                .or_else(|| app.primary_monitor().ok().flatten());

                            if let Some(mon) = monitor {
                                let screen = ScreenMetrics::from_monitor(&mon);
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
                            }
                        }

                        unsafe {
                            SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                new_x,
                                new_y,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                        }
                    }
                } else if !state.is_hovered {
                    // Xử lý sự kiện di chuột vào (Mouse Enter): Trượt ra mép (Untuck)
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
                    state.untuck();
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
                        // Debounce / Hysteresis 80ms chống rung chuột giống master
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
            unsafe { ReleaseCapture(); }
            if let Some(state) = unsafe { state_ptr.as_mut() } {
                if state.is_dragging {
                    state.is_dragging = false;

                    if state.has_dragged {
                        // Khôi phục độ mờ đầy đủ
                        state.set_layered_alpha(255);

                        // Kéo thả hoàn tất -> Kích hoạt động cơ vật lý BubbleMovementEngine để hít mép (Snap to Edge)
                        let (target_x, target_y, should_dismiss) = if let Some(app) = &state.app_handle {
                            let monitor = app
                                .get_webview_window("main")
                                .and_then(|m| m.current_monitor().ok().flatten())
                                .or_else(|| app.primary_monitor().ok().flatten());

                            let screen = monitor
                                .map(|m| ScreenMetrics::from_monitor(&m))
                                .unwrap_or(ScreenMetrics {
                                    x: 0,
                                    y: 0,
                                    width: 1920,
                                    height: 1080,
                                    scale_factor: 1.0,
                                });

                            let config = crate::movement::get_bubble_physics_config(app);
                            let all_bubbles = crate::windows::bubble::get_active_bubble_rects(app, None);

                            let snap = BubbleMovementEngine::calculate_snap(
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

                            (snap.target_x, snap.target_y, snap.should_dismiss)
                        } else {
                            let target_x = if state.x < 960 { 0 } else { 1920 - state.width as i32 };
                            (target_x, state.y, false)
                        };

                        if should_dismiss {
                            unsafe { ShowWindow(hwnd, SW_HIDE); }
                            if let Some(app) = &state.app_handle {
                                let app_clone = app.clone();
                                let idx = state.account_index;
                                tauri::async_runtime::spawn(async move {
                                    crate::commands::bubble::suspend_account_bubble_and_chat(&app_clone, idx);
                                });
                            }
                        } else {
                            // Trượt mượt mà (Spring Glide 160ms) vào mép hít với đường cong Ease-out Cubic
                            state.anim_state.start(state.x, state.y, target_x, target_y, 160);
                            unsafe {
                                SetTimer(hwnd, TIMER_ID_SPRING, 16, None);
                            }
                        }
                    } else {
                        // Thao tác Click chuột (Click to Open/Toggle Chat)
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
            if wparam == TIMER_ID_PULSE {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    state.pulse_frame += 1;
                    let total_frames = 12;
                    if state.pulse_frame >= total_frames {
                        unsafe { KillTimer(hwnd, TIMER_ID_PULSE); }
                        state.has_pulse = false;
                        state.pulse_progress = 0.0;
                        state.pulse_frame = 0;
                    } else {
                        state.has_pulse = true;
                        state.pulse_progress = state.pulse_frame as f32 / total_frames as f32;
                    }
                    state.update_layered_buffer();
                }
            } else if wparam == TIMER_ID_SPRING {
                if let Some(state) = unsafe { state_ptr.as_mut() } {
                    if let Some((cur_x, cur_y, is_finished)) = state.anim_state.step() {
                        state.x = cur_x;
                        state.y = cur_y;

                        crate::windows::native_bubble::manager::update_cached_bubble_bounds(
                            state.account_index,
                            cur_x,
                            cur_y,
                            state.width,
                            state.height,
                        );

                        unsafe {
                            SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                cur_x,
                                cur_y,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                        }

                        if is_finished {
                            unsafe { KillTimer(hwnd, TIMER_ID_SPRING); }
                            if let Some(app) = &state.app_handle {
                                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                                    cs.set_bubble_rect(
                                        state.account_index,
                                        crate::movement::BubbleRect {
                                            account_index: state.account_index,
                                            x: cur_x,
                                            y: cur_y,
                                            width: state.width as i32,
                                            height: state.height as i32,
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
                            state.tuck();
                        }
                    }
                }
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
