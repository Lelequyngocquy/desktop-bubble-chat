use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use super::dto::TransitionLockGuard;
use crate::movement::BubbleRect;
use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::ChatState;
use crate::presentation::bubble::{
    ensure_bubble_positioned_and_visible, ensure_bubble_positioned_and_visible_with_extra,
    get_bubble_window_label,
};

/// Helper shutdown: close chat webview, reclaim RAM, preserve cache on disk
pub fn suspend_account_bubble_and_chat(app: &AppHandle, account_index: u32) {
    let idx = account_index;
    let bubble_label = get_bubble_window_label(idx);
    let chat_label = get_messenger_window_label(idx);
    if let Some(state) = app.try_state::<Arc<ChatState>>() {
        state.set_bypass_close_guard(&chat_label, true);
        state.set_account_suspended(idx, true);
        state.set_chat_ready(idx, false);
        state.remove_bubble_rect(idx);
    }
    let win = app
        .get_webview_window(&chat_label)
        .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
    if let Some(cw) = win {
        let _ = cw.hide();
        let _ = cw.close();
        let _ = cw.destroy();
    }
    crate::operating_systems::current::bubble::NativeBubbleManager::global().dispatch(
        crate::operating_systems::current::bubble::NativeBubbleAction::Hide { account_index: idx }
    );
    if let Some(bw) = app.get_webview_window(&bubble_label) {
        let _ = bw.hide();
        let _ = bw.destroy();
    }
    crate::presentation::main_window::trim_current_process_working_set();
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": idx,
        "visible": false
    }));
}

/// Helper resume: restore bubble from disk cache and set snap position
pub fn resume_account_bubble_and_chat(
    app: &AppHandle,
    account_index: u32,
    committed_rects: Option<&[BubbleRect]>,
) -> Result<BubbleRect, String> {
    let idx = account_index;
    let is_connected = app
        .try_state::<Arc<ChatState>>()
        .and_then(|s| s.get_account_auth(idx))
        .map(|a| a.connected)
        .unwrap_or(false);
    if !is_connected {
        return Err(format!("Account {} is not authenticated", idx));
    }
    if let Some(state) = app.try_state::<Arc<ChatState>>() {
        state.set_account_suspended(idx, false);
        state.set_chat_ready(idx, false);
    }
    let rect = if let Some(committed) = committed_rects {
        ensure_bubble_positioned_and_visible_with_extra(app, idx, committed)?
    } else {
        ensure_bubble_positioned_and_visible(app, idx)?
    };
    let _ = app.emit("bubble-spawned", serde_json::json!({
        "accountIndex": idx,
        "x": rect.x,
        "y": rect.y
    }));
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": idx,
        "visible": true
    }));
    Ok(rect)
}

/// Command to toggle bubble window visibility for a specified account
#[tauri::command]
pub async fn toggle_bubble(
    app: AppHandle,
    visible: bool,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let state = app.try_state::<Arc<ChatState>>().map(|s| Arc::clone(&s));
    if let Some(s) = &state {
        if !s.try_acquire_transition_lock(idx) {
            println!("[toggle_bubble] Account {} is transitioning, skipping duplicate toggle", idx);
            return Ok(());
        }
    }
    let guard = TransitionLockGuard::new(state, idx);
    if visible {
        let is_connected = app
            .try_state::<Arc<ChatState>>()
            .and_then(|s| s.get_account_auth(idx))
            .map(|a| a.connected)
            .unwrap_or(false);
        if !is_connected {
            println!("[toggle_bubble] Cannot show bubble: account {} is not authenticated", idx);
            drop(guard);
            return Err(format!("Account {} is not authenticated", idx));
        }
        match resume_account_bubble_and_chat(&app, idx, None) {
            Ok(_) => {
                println!("[toggle_bubble] Account {} bubble resumed successfully", idx);
                if let Some(s) = app.try_state::<Arc<ChatState>>() {
                    s.update_user_config(&app, |c| {
                        if !c.active_bubble_accounts.contains(&idx) {
                            c.active_bubble_accounts.push(idx);
                            c.active_bubble_accounts.sort();
                        }
                    });
                }
            }
            Err(e) => eprintln!("[toggle_bubble ERROR] Failed to resume bubble for account {}: {}", idx, e),
        }
    } else {
        suspend_account_bubble_and_chat(&app, idx);
        println!("[toggle_bubble] Account {} bubble suspended successfully", idx);
        if let Some(s) = app.try_state::<Arc<ChatState>>() {
            s.update_user_config(&app, |c| {
                c.active_bubble_accounts.retain(|&x| x != idx);
            });
        }
    }
    drop(guard);
    let _ = crate::tray::update_tray_menu(&app);
    Ok(())
}

/// Helper & command to sequentially toggle all bubbles
pub async fn toggle_all_bubbles_impl(
    app: &AppHandle,
    force_visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> bool {
    let mut visible_indices: Vec<u32> = Vec::new();
    for idx in 1..=5 {
        if crate::tray::is_bubble_visible(app, idx) {
            visible_indices.push(idx);
        }
    }
    let any_visible = !visible_indices.is_empty();
    let next_visible = match force_visible {
        Some(v) => v,
        None => !any_visible,
    };
    if next_visible {
        let targets: Vec<u32> = if let Some(indices) = account_indices {
            indices.into_iter().filter(|idx| {
                app.try_state::<Arc<ChatState>>().and_then(|s| s.get_account_auth(*idx)).map(|a| a.connected).unwrap_or(false)
            }).collect()
        } else {
            crate::tray::get_available_account_indices(app)
        };
        let mut committed_rects: Vec<BubbleRect> = Vec::new();
        let mut successfully_resumed: Vec<u32> = Vec::new();
        for (i, idx) in targets.iter().enumerate() {
            match resume_account_bubble_and_chat(app, *idx, Some(&committed_rects)) {
                Ok(rect) => {
                    println!("[toggle_all_bubbles] Account {} bubble resumed successfully", idx);
                    committed_rects.push(rect);
                    successfully_resumed.push(*idx);
                }
                Err(err) => {
                    eprintln!("[toggle_all_bubbles ERROR] Failed to resume bubble for account {}: {}", idx, err);
                }
            }
            if i + 1 < targets.len() {
                tokio::time::sleep(std::time::Duration::from_millis(70)).await;
            }
        }
        if let Some(s) = app.try_state::<Arc<ChatState>>() {
            s.update_user_config(app, |c| {
                for &idx in &successfully_resumed {
                    if !c.active_bubble_accounts.contains(&idx) {
                        c.active_bubble_accounts.push(idx);
                    }
                }
                c.active_bubble_accounts.sort();
            });
        }
    } else {
        if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
            chat_state.save_previously_visible_bubbles(visible_indices);
            chat_state.update_user_config(app, |c| {
                c.active_bubble_accounts.clear();
            });
        }
        for idx in 1..=5 {
            suspend_account_bubble_and_chat(app, idx);
        }
    }
    let _ = crate::tray::update_tray_menu(app);
    next_visible
}

#[tauri::command]
pub async fn toggle_all_bubbles(
    app: AppHandle,
    visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> Result<bool, String> {
    Ok(toggle_all_bubbles_impl(&app, visible, account_indices).await)
}
