use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::tray::TrayIconBuilder;
#[cfg(not(target_os = "macos"))]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::Manager;

static IS_EXIT_DIALOG_OPEN: AtomicBool = AtomicBool::new(false);

use super::dialog::confirm_exit_dialog;
use super::helpers::{toggle_main_window, toggle_single_bubble_from_tray};
use super::menu::{build_tray_menu, update_tray_menu};
use super::TRAY_ID;

/// Initializes System Tray Icon and control context menu
pub fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app.handle())?;
    let icon = app.default_window_icon().cloned().ok_or("Failed to get app icon")?;
    let locale_str = app.handle().try_state::<Arc<crate::state::ChatState>>().and_then(|s| s.user_config.lock().ok().map(|c| c.current_locale().to_string())).unwrap_or_else(|| "en".to_string());
    let tooltip = if locale_str == "vi" { "DesktopBubbleChat - Đang chạy ngầm" } else { "DesktopBubbleChat - Running in background" };
    let tray_builder = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip(tooltip);
    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.show_menu_on_left_click(true).icon_as_template(true);
    #[cfg(not(target_os = "macos"))]
    let tray_builder = tray_builder.show_menu_on_left_click(false);
    let _tray = tray_builder
        .on_menu_event(move |app_handle, event| {
            let id_str = event.id.as_ref();
            match id_str {
                "show_home" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        toggle_main_window(&handle);
                    });
                }
                "toggle_all_bubbles" | "toggle_bubble_item" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::commands::bubble::toggle_all_bubbles_impl(&handle, None, None).await;
                        let _ = update_tray_menu(&handle);
                    });
                }
                "toggle_language" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Some(chat_state) = handle.try_state::<Arc<crate::state::ChatState>>() {
                            let current = chat_state.get_user_config().current_locale().to_string();
                            let new_lang = if current == "vi" { "en" } else { "vi" };
                            crate::commands::apply_app_language(&handle, &chat_state, new_lang);
                        }
                    });
                }
                "quit" => {
                    if IS_EXIT_DIALOG_OPEN.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                        return;
                    }
                    let locale = app_handle
                        .try_state::<Arc<crate::state::ChatState>>()
                        .map(|s| s.get_user_config().current_locale().to_string())
                        .unwrap_or_else(|| "en".to_string());
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let confirmed = tokio::task::spawn_blocking(move || confirm_exit_dialog(&locale)).await.unwrap_or(false);
                        IS_EXIT_DIALOG_OPEN.store(false, Ordering::SeqCst);
                        if confirmed {
                            if let Some(state) = handle.try_state::<Arc<crate::state::ChatState>>() {
                                state.is_explicit_exit.store(true, Ordering::SeqCst);
                                state.update_user_config(&handle, |c| {
                                    c.is_dashboard_open = true;
                                });
                            }
                            for (_, window) in handle.webview_windows() {
                                let _ = window.destroy();
                            }
                            handle.exit(0);
                        }
                    });
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
            #[cfg(not(target_os = "macos"))]
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let handle = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    toggle_main_window(&handle);
                });
            }
            #[cfg(target_os = "macos")]
            let _ = (tray, event);
        })
        .build(app)?;
    Ok(())
}
