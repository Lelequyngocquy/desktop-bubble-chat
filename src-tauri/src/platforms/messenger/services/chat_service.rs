use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::movement::BubbleMovementEngine;
use crate::platforms::messenger::windows::{
    get_messenger_webview, get_messenger_window_label, get_or_create_messenger_window,
    hide_all_other_messenger_windows,
};
use crate::state::{ChatState, MAX_ACCOUNTS};
use crate::presentation::bubble::{get_bubble_window_label, tuck_bubble};

pub struct ChatService;

impl ChatService {
    /// Docks Messenger chat window next to the corresponding bubble
    pub fn dock_chat_window(app: &AppHandle, chat_window: &tauri::WebviewWindow, account_index: u32) {
        let native_bounds = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(account_index);
        let bubble_label = get_bubble_window_label(account_index);
        let webview_bubble = app.get_webview_window(&bubble_label);
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let scale = screen.scale_factor;
        let config = crate::movement::get_bubble_physics_config(app);
        let default_bubble_w = (config.size * scale).round() as i32;
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
                (screen.x, screen.y, default_bubble_w)
            }
        } else {
            (screen.x, screen.y, default_bubble_w)
        };
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
        let max_x = (screen.x + screen.width - chat_w - margin).max(min_x);
        let final_x = target_x.clamp(min_x, max_x);
        let bubble_top = bubble_y;
        let min_y = screen.y + margin;
        let max_y = (screen.y + screen.height - chat_h - taskbar_margin).max(min_y);
        let final_y = bubble_top.clamp(min_y, max_y);
        let _ = chat_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: final_x,
            y: final_y,
        }));
        for other_idx in 1..=MAX_ACCOUNTS {
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
        let is_bubble_already_active = crate::operating_systems::current::bubble::NativeBubbleManager::global().has_bubble(idx);
        if (state.is_account_suspended(idx) || !is_bubble_already_active)
            && crate::commands::bubble::lifecycle::resume_account_bubble_and_chat(app, idx, None).is_ok() {
                state.update_user_config(app, |c| {
                    if !c.active_bubble_accounts.contains(&idx) {
                        c.active_bubble_accounts.push(idx);
                        c.active_bubble_accounts.sort();
                    }
                });
                let _ = crate::tray::update_tray_menu(app);
            }
        let chat_window = get_or_create_messenger_window(app, idx, Arc::clone(state))?;
        let is_already_visible = chat_window.is_visible().unwrap_or(false);
        if is_already_visible {
            return Ok(());
        }
        hide_all_other_messenger_windows(app, chat_window.label());
        if let Ok(curr_url) = chat_window.url() {
            let u = curr_url.to_string();
            if u.contains("facebook.com") || (!u.is_empty() && u != "about:blank" && !u.contains("messenger.com")) {
                println!("[BE - Chat] Window on URL '{}', redirecting to https://www.messenger.com/", u);
                if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                    let _ = chat_window.navigate(target);
                }
            }
        }
        state.next_chat_anim_token(idx);
        state.set_window_auth_mode(chat_window.label(), false);
        let _ = chat_window.set_decorations(false);
        let _ = chat_window.set_skip_taskbar(true);
        Self::dock_chat_window(app, &chat_window, idx);
        println!("[BE - Chat] Opened Messenger {:02} chat docked next to bubble", idx);
        crate::utils::trim_controller::cancel_auto_trim(idx);
        crate::utils::trim_controller::record_user_interaction(idx);
        crate::platforms::messenger::windows::set_webview_memory_target_level(&chat_window, false);
        state.set_chat_pinned(idx, false);
        #[cfg(target_os = "windows")]
        let opened_over_fullscreen = crate::operating_systems::windows::fullscreen_overlay::open_window_if_fullscreen(&chat_window);
        #[cfg(not(target_os = "windows"))]
        let opened_over_fullscreen = false;
        if !opened_over_fullscreen {
            chat_window.show().map_err(|e| e.to_string())?;
            chat_window.set_focus().map_err(|e| e.to_string())?;
        }
        let _ = chat_window.eval("window.__IS_AUTH_MODE__ = false; window.__isWindowFocused = true; window.__IS_PINNED__ = false; if (typeof window.__ensurePinButtonInDom === 'function') { window.__ensurePinButtonInDom(); } if (typeof window.__updatePinButtonUi === 'function') { window.__updatePinButtonUi(false); } window.dispatchEvent(new Event('focus')); window.dispatchEvent(new Event('resize')); if (typeof window.__cancelAutoUnselectCountdown === 'function') { window.__cancelAutoUnselectCountdown(); } if (typeof window.__ensureChatListVisible === 'function') { window.__ensureChatListVisible(); }");
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
        let chat_window = get_messenger_webview(app, idx);
        if let Some(chat_window) = chat_window
            && chat_window.is_visible().unwrap_or(false) {
                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                    cs.set_chat_pinned(idx, false);
                    if cs.get_chat_window_custom_size(idx).is_some()
                        && let Ok(phys_sz) = chat_window.inner_size() {
                            let scale = chat_window.scale_factor().unwrap_or(1.0);
                            let cur_w = (phys_sz.width as f64 / scale).round() as u32;
                            let cur_h = (phys_sz.height as f64 / scale).round() as u32;
                            if cur_w >= 320 && cur_h >= 450 {
                                cs.set_chat_window_custom_size(idx, Some((cur_w, cur_h)));
                            }
                        }
                }
                let _ = chat_window.eval("window.__isWindowFocused = false; window.__IS_PINNED__ = false; if (typeof window.__updatePinButtonUi === 'function') { window.__updatePinButtonUi(false); } if (document.activeElement && typeof document.activeElement.blur === 'function') { document.activeElement.blur(); } window.dispatchEvent(new Event('blur')); if (typeof window.__startAutoUnselectCountdown === 'function') { window.__startAutoUnselectCountdown(5000); }");
                #[cfg(target_os = "windows")]
                crate::operating_systems::windows::fullscreen_overlay::hide_window_with_fullscreen_cleanup(&chat_window);
                #[cfg(not(target_os = "windows"))]
                let _ = chat_window.hide();
                let hide_token = app
                    .try_state::<Arc<ChatState>>()
                    .map(|cs| cs.next_chat_anim_token(idx))
                    .unwrap_or(0);
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    let should_abort = || {
                        app_handle.try_state::<Arc<ChatState>>().is_some_and(|cs| {
                            cs.is_explicit_exit.load(std::sync::atomic::Ordering::SeqCst)
                                || (hide_token != 0 && !cs.is_chat_anim_token_valid(idx, hide_token))
                        })
                    };
                    std::thread::sleep(std::time::Duration::from_millis(60));
                    if should_abort() {
                        return;
                    }
                    let _ = app_handle.emit("chat-visibility-changed", serde_json::json!({
                        "accountIndex": idx,
                        "visible": false,
                        "windowLabel": label
                    }));
                    std::thread::sleep(std::time::Duration::from_millis(220));
                    if should_abort() {
                        return;
                    }
                    if let Some(cw) = get_messenger_webview(&app_handle, idx)
                        && !cw.is_visible().unwrap_or(false) {
                            tuck_bubble(&app_handle, Some(idx));
                        }
                });
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
        println!("[BE - Messenger] Reloading Messenger session {:02}", idx);
        if let Some(chat_window) = get_messenger_webview(app, idx) {
            let _ = chat_window.eval("window.location.reload();");
        }
        Ok(())
    }
}
