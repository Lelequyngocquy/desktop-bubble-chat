use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::operating_systems::current::CursorTracker;
use crate::platforms::messenger::services::ChatService;
use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::{ChatState, MAX_ACCOUNTS};
use crate::presentation::bubble::get_bubble_window_label;

/// Daemon thread: Hover 0.1s over bubble to open chat, auto-close when cursor leaves both chat and bubble
pub fn start_hover_and_autoclose_monitor(app: AppHandle, chat_state: Arc<ChatState>) {
    std::thread::spawn(move || {
        let mut current_hover_bubble: Option<u32> = None;
        let mut hover_start: Option<Instant> = None;
        let mut already_opened_for_hover = false;
        let mut outside_open_chat_start: Option<Instant> = None;
        loop {
            if chat_state.is_explicit_exit.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(30));
            if chat_state.is_explicit_exit.load(Ordering::SeqCst) {
                break;
            }
            let lbutton_down = CursorTracker::is_lbutton_down();
            let mut detected_hover_bubble: Option<u32> = None;
            if !lbutton_down {
                for idx in 1..=MAX_ACCOUNTS {
                    if chat_state.is_account_suspended(idx) {
                        continue;
                    }
                    let label = get_bubble_window_label(idx);
                    if let Some(bubble_win) = app.get_webview_window(&label) {
                        if bubble_win.is_visible().unwrap_or(false) && CursorTracker::is_cursor_over_window(&bubble_win) {
                            detected_hover_bubble = Some(idx);
                            break;
                        }
                    } else if crate::operating_systems::current::bubble::is_cursor_over_native_bubble(idx) {
                        detected_hover_bubble = Some(idx);
                        break;
                    }
                }
            }
            if let Some(hovered_idx) = detected_hover_bubble {
                if current_hover_bubble == Some(hovered_idx) {
                    if !already_opened_for_hover
                        && let Some(start) = hover_start {
                            let threshold_ms = if chat_state.is_chat_ready(hovered_idx) { 100 } else { 400 };
                            if start.elapsed() >= Duration::from_millis(threshold_ms) {
                                already_opened_for_hover = true;
                                let app_clone = app.clone();
                                let state_clone = Arc::clone(&chat_state);
                                tauri::async_runtime::spawn(async move {
                                    let _ = ChatService::open_chat(&app_clone, &state_clone, hovered_idx);
                                });
                            }
                        }
                } else {
                    current_hover_bubble = Some(hovered_idx);
                    hover_start = Some(Instant::now());
                    already_opened_for_hover = false;
                    crate::operating_systems::current::bubble::untuck_native_bubble(&app, hovered_idx);
                }
            } else {
                current_hover_bubble = None;
                hover_start = None;
                already_opened_for_hover = false;
            }
            let mut active_chat_info: Option<(u32, tauri::WebviewWindow)> = None;
            for idx in 1..=MAX_ACCOUNTS {
                if chat_state.is_account_suspended(idx) {
                    continue;
                }
                let label = get_messenger_window_label(idx);
                if let Some(chat_win) = app.get_webview_window(&label)
                    && chat_win.is_visible().unwrap_or(false)
                    && !chat_state.is_window_in_auth_mode(&label) {
                        active_chat_info = Some((idx, chat_win));
                        break;
                    }
            }
            if !lbutton_down {
                if let Some((active_idx, chat_win)) = active_chat_info {
                    let bubble_label = get_bubble_window_label(active_idx);
                    let over_chat = CursorTracker::is_cursor_over_window(&chat_win);
                    let over_bubble = app
                        .get_webview_window(&bubble_label)
                        .map(|b| CursorTracker::is_cursor_over_window(&b))
                        .unwrap_or_else(|| crate::operating_systems::current::bubble::is_cursor_over_native_bubble(active_idx));
                    let over_action_dock = crate::operating_systems::current::bubble::is_cursor_over_native_action_dock();
                    let is_dialog_open = chat_state.is_any_dialog_active();
                    if over_chat || over_bubble || over_action_dock || chat_state.is_chat_pinned(active_idx) || is_dialog_open {
                        outside_open_chat_start = None;
                    } else {
                        match outside_open_chat_start {
                            None => {
                                outside_open_chat_start = Some(Instant::now());
                            }
                            Some(start_time) => {
                                if start_time.elapsed() >= Duration::from_millis(300) {
                                    if !chat_state.is_chat_pinned(active_idx) && !is_dialog_open {
                                        let app_clone = app.clone();
                                        tauri::async_runtime::spawn(async move {
                                            let _ = ChatService::hide_chat(&app_clone, active_idx);
                                            crate::operating_systems::current::bubble::tuck_native_bubble(&app_clone, active_idx);
                                        });
                                    }
                                    outside_open_chat_start = None;
                                }
                            }
                        }
                    }
                } else {
                    outside_open_chat_start = None;
                }
            } else {
                outside_open_chat_start = None;
            }
        }
    });
}
