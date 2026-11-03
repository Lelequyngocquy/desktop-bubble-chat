pub mod commands;
pub mod movement;
pub mod platforms;
pub mod state;
pub mod tray;
pub mod utils;
pub mod windows;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use state::ChatState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let chat_state = Arc::new(ChatState::default());
    let chat_state_clone = Arc::clone(&chat_state);

    let app = tauri::Builder::default()
        .manage(chat_state)
        .manage(movement::BubbleConfigState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
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
            commands::get_active_bubble_accounts,
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
            commands::report_incoming_notification,
            commands::get_recent_messages,
            commands::report_new_message,
            commands::sync_recent_conversations,
            commands::set_account_muted,
            commands::update_unread_badge,
            commands::set_badge_count,
            commands::get_unread_count,
            commands::get_all_unread_counts,
            commands::toggle_autostart,
            commands::is_autostart_enabled,
            commands::get_system_username,
            commands::open_external_url,
            commands::minimize_main_window,
            commands::toggle_maximize_main_window,
            commands::hide_main_window,
            commands::show_main_window,
            commands::update_tray_menu_status,
            commands::get_user_settings,
            commands::save_user_settings,
            commands::exit_app,
            commands::set_always_on_top,
            commands::log_frontend,
            commands::toggle_native_bubble_preview,
            commands::trigger_native_bubble_pulse_preview,
            commands::set_native_bubble_badge_preview,
            commands::tuck_native_bubble_preview,
            commands::untuck_native_bubble_preview
        ])
        .setup(move |app| {
            // 1. Cấu hình System Tray Menu & Icon (cần &mut app)
            if let Err(err) = tray::setup_system_tray(app) {
                eprintln!("[Tray Error] Failed to setup system tray: {}", err);
            }

            let handle = app.handle().clone();
            let initial_config = utils::load_user_cache(&handle);
            chat_state_clone.init_user_config(initial_config.clone());

            // 2. Đồng bộ kích thước Bong bóng từ Cache vào BubbleConfigState TRƯỚC KHI spawn bất kỳ bubble nào
            let target_bubble_size = initial_config.bubble_size_px();
            if let Some(state) = app.try_state::<movement::BubbleConfigState>() {
                if let Ok(mut guard) = state.0.write() {
                    guard.size = target_bubble_size;
                }
            }

            // 3. Áp dụng các cài đặt muted vào chat_state
            for &idx in &initial_config.muted_accounts {
                chat_state_clone.set_account_muted(idx, true);
            }

            // 4. Áp dụng Always-on-top
            chat_state_clone.is_always_on_top.store(initial_config.is_always_on_top, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            crate::windows::native_bubble::set_native_bubble_always_on_top(initial_config.is_always_on_top);

            // 5. Khôi phục các Bong bóng người dùng đã cấu hình bật (KHÔNG tắt bubble một cách mặc định!)
            println!("[Startup] Khôi phục các Bong bóng đang bật từ cache: {:?}", initial_config.active_bubble_accounts);
            let mut committed_rects: Vec<movement::BubbleRect> = Vec::new();
            for &idx in &initial_config.active_bubble_accounts {
                match commands::bubble::resume_account_bubble_and_chat(&handle, idx, Some(&committed_rects)) {
                    Ok(rect) => {
                        println!("[Startup] Đã khôi phục thành công Bong bóng {:02} tại ({}, {})", idx, rect.x, rect.y);
                        committed_rects.push(rect);
                    }
                    Err(err) => {
                        eprintln!("[Startup ERROR] Khôi phục bubble {} thất bại: {}", idx, err);
                    }
                }
            }

            // 5. Cấu hình và điều khiển cửa sổ chính (Home Dashboard)
            windows::main_window::setup_main_window(&handle);
            if initial_config.is_dashboard_open {
                println!("[Startup] Dashboard được cấu hình BẬT -> Hiển thị Dashboard");
                let _ = windows::main_window::get_or_create_main_window(&handle);
            } else {
                println!("[Startup] Dashboard được cấu hình TẮT -> Giữ Dashboard ở chế độ ngủ đông (0 MB RAM)");
                windows::main_window::destroy_or_hibernate_main_window(&handle);
            }

            // 6. Quản lý sự kiện trên cửa sổ chat Messenger (Auto-hide on blur vs Auth Mode)
            platforms::messenger::windows::setup_chat_window(&handle, Arc::clone(&chat_state_clone));

            // 7. Luồng giám sát trạng thái đăng nhập và URL của Messenger theo thời gian thực
            platforms::messenger::daemons::start_auth_watcher(handle.clone(), Arc::clone(&chat_state_clone));

            // 8. Luồng giám sát toàn cục: Hover 0.1s trên Bubble để mở Chat, rời khỏi thì tự động đóng
            platforms::messenger::daemons::start_hover_and_autoclose_monitor(handle.clone(), Arc::clone(&chat_state_clone));

            // 9. Phím tắt toàn cục (Global Shortcut: Ctrl+Shift+M)
            if let Ok(shortcut) = "Ctrl+Shift+M".parse::<Shortcut>() {
                if let Err(err) = app.global_shortcut().register(shortcut) {
                    eprintln!("[Global Shortcut] Failed to register Ctrl+Shift+M: {}", err);
                }
            }

            // 10. Cập nhật menu System Tray khớp 1:1 với config vừa nạp
            let _ = tray::update_tray_menu(&handle);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
            let is_explicit = app_handle
                .try_state::<Arc<ChatState>>()
                .map(|s| s.is_explicit_exit.load(Ordering::SeqCst))
                .unwrap_or(false);

            if !is_explicit {
                // Ngăn chặn Tauri tự động thoát tiến trình Backend khi toàn bộ cửa sổ đã bị đóng/hủy
                println!("[Tauri Lifecycle] Chặn thoát tự động: Giữ Rust Host & System Tray tiếp tục chạy ngầm (Exit code: {:?})", code);
                api.prevent_exit();
            } else {
                println!("[Tauri Lifecycle] Người dùng xác nhận thoát hoàn toàn ứng dụng (Exit code: {:?})", code);
            }
        }
    });
}
