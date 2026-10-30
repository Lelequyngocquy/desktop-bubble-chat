use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::platforms::messenger::services::AuthService;
use crate::state::ChatState;

/// Command mở cửa sổ popup đăng nhập Messenger độc lập cho tài khoản chỉ định
#[tauri::command]
pub async fn open_messenger_auth(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    AuthService::open_auth(&app, &state, idx).await
}

/// Command kiểm tra trạng thái đăng nhập thực tế của tài khoản Messenger
#[tauri::command]
pub fn check_messenger_auth(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<serde_json::Value, String> {
    let idx = account_index.unwrap_or(1);
    AuthService::check_auth(&app, &state, idx)
}

/// Command nhận báo cáo trạng thái xác thực từ webview và đồng bộ sang UI
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

/// Command ngắt kết nối Messenger, điều hướng về login và ẩn cửa sổ
#[tauri::command]
pub fn disconnect_messenger(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    AuthService::disconnect(&app, &state, idx)
}
