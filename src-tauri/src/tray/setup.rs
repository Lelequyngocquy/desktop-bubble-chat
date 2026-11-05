use std::sync::Arc;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use super::dialog::confirm_exit_dialog;
use super::helpers::{toggle_main_window, toggle_single_bubble_from_tray};
use super::menu::{build_tray_menu, update_tray_menu};
use super::TRAY_ID;

/// Initializes System Tray Icon and control context menu
pub fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(&app.handle())?;
    let icon = app.default_window_icon().cloned().ok_or("Failed to get app icon")?;
    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            let id_str = event.id.as_ref();
            match id_str {
                "show_home" => {
                    toggle_main_window(app_handle);
                }
                "toggle_all_bubbles" | "toggle_bubble_item" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::commands::bubble::toggle_all_bubbles_impl(&handle, None, None).await;
                        let _ = update_tray_menu(&handle);
                    });
                }
                "quit" => {
                    let locale = app_handle
                        .try_state::<Arc<crate::state::ChatState>>()
                        .map(|s| s.get_user_config().current_locale().to_string())
                        .unwrap_or_else(|| "en".to_string());
                    if confirm_exit_dialog(&locale) {
                        if let Some(state) = app_handle.try_state::<Arc<crate::state::ChatState>>() {
                            state.is_explicit_exit.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        app_handle.exit(0);
                    }
                }
                id if id.starts_with("toggle_bubble_") => {
                    if let Ok(idx) = id.trim_start_matches("toggle_bubble_").parse::<u32>() {
                        let handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            toggle_single_bubble_from_tray(&handle, idx).await;
                            let _ = update_tray_menu(&handle);
                        });
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app_handle = tray.app_handle();
                toggle_main_window(app_handle);
            }
        })
        .build(app)?;
    Ok(())
}
