pub mod commands;
pub mod movement;
pub mod platforms;
pub mod state;
pub mod tray;
pub mod utils;
pub mod windows;

use std::sync::Arc;
use state::ChatState;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let chat_state = Arc::new(ChatState::default());
    let chat_state_clone = Arc::clone(&chat_state);

    tauri::Builder::default()
        .manage(chat_state)
        .manage(movement::BubbleConfigState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app_handle, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Ok(target) = "Ctrl+Shift+M".parse::<Shortcut>() {
                            if shortcut == &target {
                                let handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    commands::toggle_all_bubbles_impl(&handle, None, None).await;
                                });
                            }
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::toggle_bubble,
            commands::toggle_all_bubbles,
            commands::snap_bubble,
            commands::get_bubble_snap_coords,
            commands::get_bubble_default_coords,
            commands::get_bubble_hover_coords,
            commands::get_bubble_position,
            commands::get_all_bubble_positions,
            commands::set_bubble_config,
            commands::toggle_chat,
            commands::open_chat,
            commands::preload_chat,
            commands::open_thread,
            commands::open_messenger_auth,
            commands::check_messenger_auth,
            commands::disconnect_messenger,
            commands::refresh_messenger,
            commands::close_messenger_window,
            commands::clear_account_storage,
            commands::report_auth_status,
            commands::report_new_message,
            commands::sync_recent_conversations,
            commands::update_unread_badge,
            commands::set_badge_count,
            commands::toggle_autostart,
            commands::is_autostart_enabled,
            commands::get_system_username,
            commands::open_external_url,
            commands::exit_app,
            commands::set_always_on_top,
            commands::log_frontend
        ])
        .setup(move |app| {
            // 1. Cấu hình System Tray Menu & Icon
            if let Err(err) = tray::setup_system_tray(app) {
                eprintln!("[Tray Error] Failed to setup system tray: {}", err);
            }

            // 2. Chặn việc đóng cửa sổ chính (Home Page) -> Ẩn về khay hệ thống, giữ chat heads chạy ngầm
            windows::main_window::setup_main_window(&app.handle());
            #[cfg(target_os = "windows")]
            windows::bubble::setup_bubble_window(&app.handle());

            // 3. Quản lý sự kiện trên cửa sổ chat Messenger (Auto-hide on blur vs Auth Mode)
            platforms::messenger::windows::setup_chat_window(&app.handle(), Arc::clone(&chat_state_clone));

            // 4. Luồng giám sát trạng thái đăng nhập và URL của Messenger theo thời gian thực
            platforms::messenger::daemons::start_auth_watcher(app.handle().clone(), Arc::clone(&chat_state_clone));

            // 5. Luồng giám sát toàn cục: Hover 0.1s trên Bubble để mở Chat, rời khỏi thì tự động đóng
            platforms::messenger::daemons::start_hover_and_autoclose_monitor(app.handle().clone(), Arc::clone(&chat_state_clone));

            // 6. Phím tắt toàn cục (Global Shortcut: Ctrl+Shift+M)
            if let Ok(shortcut) = "Ctrl+Shift+M".parse::<Shortcut>() {
                if let Err(err) = app.global_shortcut().register(shortcut) {
                    eprintln!("[Global Shortcut] Failed to register Ctrl+Shift+M: {}", err);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
