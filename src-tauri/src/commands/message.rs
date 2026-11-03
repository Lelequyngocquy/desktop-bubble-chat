use std::sync::Arc;
use tauri::State;

use crate::state::{ChatState, IncomingMessage};

/// Command truy vấn danh sách tin nhắn gần nhất đã lưu trong ChatState (Giai đoạn 2)
#[tauri::command]
pub fn get_recent_messages(
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<Vec<IncomingMessage>, String> {
    Ok(state.get_recent_messages(account_index))
}
