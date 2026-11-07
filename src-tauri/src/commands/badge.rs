use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::state::ChatState;

// Command to update unread badge count from injected script and emit event
#[tauri::command]
pub fn update_unread_badge(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    count: u32,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    state.set_unread_count(idx, count);
    if count == 0 {
        state.clear_recent_messages_for_account(idx);
    }
    crate::operating_systems::current::bubble::set_native_bubble_unread_count(idx, count);
    let _ = app.emit("badge-updated", serde_json::json!({
        "accountIndex": idx,
        "count": count
    }));
    Ok(())
}

// Command to set badge count directly
#[tauri::command]
pub fn set_badge_count(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    count: u32,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    state.set_unread_count(idx, count);
    if count == 0 {
        state.clear_recent_messages_for_account(idx);
    }
    crate::operating_systems::current::bubble::set_native_bubble_unread_count(idx, count);
    let _ = app.emit("badge-updated", serde_json::json!({
        "accountIndex": idx,
        "count": count
    }));
    Ok(())
}

/// Command to query unread count of a specific account (SSOT)
#[tauri::command]
pub fn get_unread_count(
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<u32, String> {
    let idx = account_index.unwrap_or(1);
    Ok(state.get_unread_count(idx))
}

/// Command to query all unread counts across all accounts from backend SSOT
#[tauri::command]
pub fn get_all_unread_counts(
    state: State<'_, Arc<ChatState>>,
) -> Result<std::collections::HashMap<u32, u32>, String> {
    if let Ok(map) = state.unread_counts.lock() {
        Ok(map.clone())
    } else {
        Ok(std::collections::HashMap::new())
    }
}
