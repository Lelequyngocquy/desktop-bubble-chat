use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::movement::get_bubble_physics_config;
use crate::state::ChatState;
use super::model::DockActionId;

/// Executes a native ActionDock droplet click through the unified Rust SSOT core functions
pub fn execute_dock_action(app: &AppHandle, account_index: u32, action: DockActionId) {
    crate::utils::trim_controller::record_user_interaction(account_index);
    match action {
        DockActionId::Pin => {
            if let Some(state) = app.try_state::<Arc<ChatState>>() {
                let _ = crate::commands::messenger::chat::apply_chat_pinned(app, &state, account_index, None);
            }
        }
        DockActionId::AllowResize => {
            if let Some(state) = app.try_state::<Arc<ChatState>>() {
                let next_custom = state.get_chat_window_custom_size(account_index).is_none();
                let _ = crate::commands::messenger::chat::apply_chat_window_size_mode(app, &state, account_index, next_custom, None, None);
            }
        }
        DockActionId::PositionMode => {
            let config = get_bubble_physics_config(app);
            let next_mode = if config.is_free_position { "auto_tuck" } else { "free" };
            let _ = crate::commands::bubble::motion::apply_bubble_position_mode(app, next_mode);
        }
        DockActionId::Mute => {
            if let Some(state) = app.try_state::<Arc<ChatState>>() {
                let next_muted = !state.is_account_muted(account_index);
                let _ = crate::commands::messenger::chat::apply_account_muted(app, &state, account_index, next_muted);
            }
        }
        DockActionId::HideBubble => {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                crate::commands::bubble::lifecycle::hide_account_bubble_ssot(app_clone, account_index).await;
            });
        }
    }
}
