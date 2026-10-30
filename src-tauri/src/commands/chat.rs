use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::platforms::messenger::services::{ChatService, StorageService};
use crate::state::ChatState;

/// Command mở trực tiếp khung chat cạnh bong bóng nổi cho tài khoản chỉ định
#[tauri::command]
pub async fn open_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::open_chat(&app, &state, idx)
}

/// Command bật/tắt khung chat Messenger khi người dùng click vào bong bóng nổi
#[tauri::command]
pub async fn toggle_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    ChatService::toggle_chat(&app, &state, idx)
}

/// Command tải lại trang Messenger cho tài khoản chỉ định
#[tauri::command]
pub fn refresh_messenger(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::refresh(&app, idx)
}

/// Command đóng và hủy hẳn cửa sổ webview của tài khoản bị xóa, xóa sạch cache/storage
#[tauri::command]
pub fn close_messenger_window(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command xóa toàn bộ bộ nhớ đệm, cookies và storage của một tài khoản cụ thể trên ổ cứng
#[tauri::command]
pub fn clear_account_storage(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    StorageService::clean_account_session(&app, &state, idx)
}
