use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::platforms::messenger::services::AuthService;
use crate::state::ChatState;

/// Command to open independent Messenger auth popup for specified account
#[tauri::command]
pub async fn open_messenger_auth(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    AuthService::open_auth(&app, &state, idx).await
}

/// Command to check real-time auth status for Messenger account
#[tauri::command]
pub fn check_messenger_auth(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<serde_json::Value, String> {
    let idx = account_index.unwrap_or(1);
    AuthService::check_auth(&app, &state, idx)
}

/// Command to receive auth status report from webview and sync to UI
#[tauri::command]
pub fn report_auth_status(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    connected: bool,
    name: Option<String>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    AuthService::report_status(&app, &state, connected, name, idx)
}

/// Command to disconnect Messenger, navigate to login, and hide window
#[tauri::command]
pub fn disconnect_messenger(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    AuthService::disconnect(&app, &state, idx)
}
