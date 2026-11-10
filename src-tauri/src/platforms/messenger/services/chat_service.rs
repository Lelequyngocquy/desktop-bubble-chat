use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::movement::BubbleMovementEngine;
use crate::platforms::messenger::constants::MESSENGER_INJECT_SCRIPT;
use crate::platforms::messenger::windows::{
    get_messenger_window_label, get_or_create_messenger_window, hide_all_other_messenger_windows,
};
use crate::state::ChatState;
use crate::presentation::bubble::{get_bubble_window_label, tuck_bubble};

pub struct ChatService;

impl ChatService {
    /// Docks Messenger chat window next to the corresponding bubble
    pub fn dock_chat_window(app: &AppHandle, chat_window: &tauri::WebviewWindow, account_index: u32) {
        #[cfg(target_os = "windows")]
        let native_bounds = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(account_index);
        #[cfg(not(target_os = "windows"))]
        let native_bounds: Option<(i32, i32, u32, u32)> = None;
        let bubble_label = get_bubble_window_label(account_index);
        let webview_bubble = app.get_webview_window(&bubble_label);
        let (bubble_x, bubble_y, bubble_w) = if let Some((bx, by, bw, _)) = native_bounds {
            (bx, by, bw as i32)
        } else if let Some(bw) = &webview_bubble {
            let pos = bw.outer_position().unwrap_or_default();
            let size = bw.outer_size().unwrap_or_default();
            (pos.x, pos.y, size.width as i32)
        } else if let Some(cs) = app.try_state::<Arc<ChatState>>() {
            if let Some(rect) = cs.get_bubble_rect(account_index) {
                (rect.x, rect.y, rect.width)
            } else {
                (0, 0, 56)
            }
        } else {
            (0, 0, 56)
        };
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let scale = screen.scale_factor;
        let config = crate::movement::get_bubble_physics_config(app);
        let (custom_w, custom_h) = app.try_state::<Arc<ChatState>>()
            .and_then(|cs| cs.get_chat_window_custom_size(account_index))
            .map(|(w, h)| (w as f64, h as f64))
            .unwrap_or((config.chat_width, config.chat_height));
        let chat_w = (custom_w * scale).round() as i32;
        let chat_h = (custom_h * scale).round() as i32;
        let gap = (config.chat_gap * scale).round() as i32;
        let margin = (config.margin * scale) as i32;
        let taskbar_margin = (config.taskbar_margin * scale) as i32;
        let _ = chat_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: chat_w as u32,
            height: chat_h as u32,
        }));
        let untucked_pos = if config.is_free_position {
            tauri::PhysicalPosition { x: bubble_x, y: bubble_y }
        } else {
            let pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                bubble_x,
                bubble_y,
                true,
            );
            if let Some(bw) = &webview_bubble {
                let _ = bw.set_position(tauri::Position::Physical(pos));
            } else {
                crate::operating_systems::current::bubble::untuck_native_bubble(app, account_index);
            }
            pos
        };
        let mid_x = screen.x + screen.width / 2;
        let target_x = if untucked_pos.x + bubble_w / 2 >= mid_x {
            untucked_pos.x - chat_w - gap
        } else {
            untucked_pos.x + bubble_w + gap
        };
        let min_x = screen.x + margin;
        let max_x = screen.x + screen.width - chat_w - margin;
        let final_x = target_x.clamp(min_x, max_x);
        let bubble_top = bubble_y;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - chat_h - taskbar_margin).max(min_y);
        let final_y = bubble_top.clamp(min_y, max_y);
        let _ = chat_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: final_x,
            y: final_y,
        }));
        for other_idx in 1..=5 {
            if other_idx != account_index {
                tuck_bubble(app, Some(other_idx));
            }
        }
    }

    /// Opens Messenger chat window for specified account
    pub fn open_chat(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let is_connected = state.get_account_auth(idx).map(|a| a.connected).unwrap_or(false);
        if !is_connected {
            return Err(format!("Account {} is not authenticated", idx));
        }
        let is_bubble_already_active = {
            #[cfg(target_os = "windows")]
            {
                crate::operating_systems::current::bubble::NativeBubbleManager::global().has_bubble(idx)
            }
            #[cfg(not(target_os = "windows"))]
            {
                let bubble_label = get_bubble_window_label(idx);
                app.get_webview_window(&bubble_label)
                    .map(|w| w.is_visible().unwrap_or(false))
                    .unwrap_or(false)
            }
        };
        if state.is_account_suspended(idx) || !is_bubble_already_active {
            if crate::commands::bubble::lifecycle::resume_account_bubble_and_chat(app, idx, None).is_ok() {
                state.update_user_config(app, |c| {
                    if !c.active_bubble_accounts.contains(&idx) {
                        c.active_bubble_accounts.push(idx);
                        c.active_bubble_accounts.sort();
                    }
                });
                let _ = crate::tray::update_tray_menu(app);
            }
        }
        let chat_window = get_or_create_messenger_window(app, idx, Arc::clone(state))?;
        let is_already_visible = chat_window.is_visible().unwrap_or(false);
        if is_already_visible {
            return Ok(());
        }
        hide_all_other_messenger_windows(app, &chat_window.label());
        if let Ok(curr_url) = chat_window.url() {
            let u = curr_url.to_string();
            if u.contains("facebook.com") || (!u.is_empty() && u != "about:blank" && !u.contains("messenger.com")) {
                println!("[BE - Chat] Window on URL '{}', redirecting to https://www.messenger.com/", u);
                if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                    let _ = chat_window.navigate(target);
                }
            }
        }
        state.set_window_auth_mode(&chat_window.label(), false);
        let _ = chat_window.set_decorations(false);
        let _ = chat_window.set_skip_taskbar(true);
        Self::dock_chat_window(app, &chat_window, idx);
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);
        state.clear_unread_count(idx);
        state.clear_recent_messages_for_account(idx);
        crate::operating_systems::current::bubble::set_native_bubble_unread_count(idx, 0);
        let _ = app.emit("badge-updated", serde_json::json!({
            "accountIndex": idx,
            "count": 0
        }));
        println!("[BE - Chat] Opened Messenger {:02} chat docked next to bubble", idx);
        crate::utils::trim_controller::cancel_auto_trim(idx);
        crate::utils::trim_controller::record_user_interaction(idx);
        crate::platforms::messenger::windows::set_webview_memory_target_level(&chat_window, false);
        chat_window.show().map_err(|e| e.to_string())?;
        chat_window.set_focus().map_err(|e| e.to_string())?;
        let _ = chat_window.eval("window.__isWindowFocused = true; window.dispatchEvent(new Event('focus'));");
        let _ = app.emit("bubble-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": true
        }));
        let _ = app.emit("chat-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": true,
            "windowLabel": chat_window.label()
        }));
        Ok(())
    }

    /// Hides Messenger chat window and tucks bubble to edge
    pub fn hide_chat(
        app: &AppHandle,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        let chat_window = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
        if let Some(chat_window) = chat_window {
            if chat_window.is_visible().unwrap_or(false) {
                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                    if cs.get_chat_window_custom_size(idx).is_some() {
                        if let Ok(phys_sz) = chat_window.inner_size() {
                            let scale = chat_window.scale_factor().unwrap_or(1.0);
                            let cur_w = (phys_sz.width as f64 / scale).round() as u32;
                            let cur_h = (phys_sz.height as f64 / scale).round() as u32;
                            if cur_w >= 320 && cur_h >= 450 {
                                cs.set_chat_window_custom_size(idx, Some((cur_w, cur_h)));
                            }
                        }
                    }
                }
                let _ = chat_window.eval("window.__isWindowFocused = false; if (document.activeElement && typeof document.activeElement.blur === 'function') { document.activeElement.blur(); } window.dispatchEvent(new Event('blur'));");
                let _ = chat_window.hide();
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(60));
                    let _ = app_handle.emit("chat-visibility-changed", serde_json::json!({
                        "accountIndex": idx,
                        "visible": false,
                        "windowLabel": label
                    }));
                    std::thread::sleep(std::time::Duration::from_millis(220));
                    if let Some(cw) = app_handle
                        .get_webview_window(&label)
                        .or_else(|| if idx == 1 { app_handle.get_webview_window("chat-messenger") } else { None })
                    {
                        if !cw.is_visible().unwrap_or(false) {
                            tuck_bubble(&app_handle, Some(idx));
                        }
                    }
                });
            }
        }
        Ok(())
    }

    /// Toggles Messenger chat window
    pub fn toggle_chat(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<bool, String> {
        let idx = account_index;
        let chat_window = get_or_create_messenger_window(app, idx, Arc::clone(state))?;
        let is_visible = chat_window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = Self::hide_chat(app, idx);
            Ok(false)
        } else {
            let _ = Self::open_chat(app, state, idx);
            Ok(true)
        }
    }

    /// Reloads Messenger page for specified account
    pub fn refresh(
        app: &AppHandle,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!("[BE - Messenger] Reloading Messenger session {:02}", idx);
        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
        if let Some(chat_window) = win {
            let _ = chat_window.eval("window.location.reload();");
        }
        Ok(())
    }
}
