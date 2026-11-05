use std::sync::Arc;
use tauri::State;

use crate::state::{ChatState, IncomingMessage};

/// Command to query recent messages stored in ChatState
#[tauri::command]
pub fn get_recent_messages(
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<Vec<IncomingMessage>, String> {
    Ok(state.get_recent_messages(account_index))
}
