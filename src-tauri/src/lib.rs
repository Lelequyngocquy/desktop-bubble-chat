pub mod commands;
pub mod core;
pub mod extensions;
pub mod movement;
#[path = "operating-systems/mod.rs"]
pub mod operating_systems;
pub mod platforms;
pub mod state;
pub mod tray;
pub mod utils;
pub mod presentation;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use state::ChatState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    unsafe {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            crate::platforms::messenger::constants::CHROMIUM_BROWSER_ARGS,
        );
    }
    #[cfg(target_os = "linux")]
    unsafe {
        if std::env::var("GDK_BACKEND").is_err() {
            std::env::set_var("GDK_BACKEND", "x11,wayland");
        }
    }
    let chat_state = Arc::new(ChatState::default());
    let chat_state_clone = Arc::clone(&chat_state);
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            println!("\x1b[33m[SingleInstance] Second instance launch detected -> Focusing primary instance...\x1b[0m");
            let handle = app.clone();
            let _ = crate::presentation::main_window::get_or_create_main_window(&handle);
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.unminimize();
                let _ = main_win.show();
                let _ = main_win.set_focus();
                #[cfg(target_os = "windows")]
                if let Ok(hwnd) = main_win.hwnd() {
                    unsafe {
                        #[link(name = "user32")]
                        unsafe extern "system" {
                            fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
                            fn BringWindowToTop(hWnd: *mut std::ffi::c_void) -> i32;
                            fn SetForegroundWindow(hWnd: *mut std::ffi::c_void) -> i32;
                        }
                        ShowWindow(hwnd.0 as *mut std::ffi::c_void, 9);
                        BringWindowToTop(hwnd.0 as *mut std::ffi::c_void);
                        SetForegroundWindow(hwnd.0 as *mut std::ffi::c_void);
                    }
                }
            }
        }))
        .manage(chat_state)
        .manage(movement::BubbleConfigState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app_handle, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let is_match = "Ctrl+Shift+M".parse::<Shortcut>().map(|s| &s == shortcut).unwrap_or(false)
                            || "CommandOrControl+Shift+M".parse::<Shortcut>().map(|s| &s == shortcut).unwrap_or(false)
                            || "Super+Shift+M".parse::<Shortcut>().map(|s| &s == shortcut).unwrap_or(false);
                        if is_match {
                            let handle = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                commands::toggle_all_bubbles_impl(&handle, None, None).await;
                            });
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
            commands::set_bubble_position_mode,
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
            commands::open_profiles_folder,
            commands::minimize_main_window,
            commands::toggle_maximize_main_window,
            commands::hide_main_window,
            commands::show_main_window,
            commands::update_tray_menu_status,
            commands::get_user_settings,
            commands::save_user_settings,
            commands::set_app_language,
            commands::set_auto_trim_enabled,
            commands::set_theme_mode,
            commands::exit_app,
            commands::restart_app,
            commands::set_always_on_top,
            commands::log_frontend,
            commands::toggle_native_bubble_preview,
            commands::trigger_native_bubble_pulse_preview,
            commands::set_native_bubble_badge_preview,
            commands::tuck_native_bubble_preview,
            commands::untuck_native_bubble_preview,
            commands::get_performance_snapshot,
            commands::trim_system_memory,
            commands::get_profile_extensions,
            commands::load_profile_extension,
            commands::toggle_profile_extension,
            commands::remove_profile_extension,
            commands::open_profile_extensions_folder,
            commands::reload_messenger_window,
            commands::set_chat_window_size_config
        ])
        .setup(move |app| {
            if let Err(err) = tray::setup_system_tray(app) {
                eprintln!("[Tray Error] Failed to setup system tray: {}", err);
            }
            let handle = app.handle().clone();
            let initial_config = utils::load_user_cache(&handle);
            chat_state_clone.init_user_config(initial_config.clone());
            let target_bubble_size = initial_config.bubble_size_px();
            if let Some(state) = app.try_state::<movement::BubbleConfigState>() {
                if let Ok(mut guard) = state.0.write() {
                    guard.size = target_bubble_size;
                    guard.is_free_position = initial_config.is_free_position_mode();
                }
            }
            for &idx in &initial_config.muted_accounts {
                chat_state_clone.set_account_muted(idx, true);
            }
            chat_state_clone.is_always_on_top.store(initial_config.is_always_on_top, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            crate::operating_systems::current::bubble::set_native_bubble_always_on_top(initial_config.is_always_on_top);
            presentation::main_window::setup_main_window(&handle);
            println!("[Startup] Restoring active bubbles from cache: {:?}", initial_config.active_bubble_accounts);
            let mut committed_rects: Vec<movement::BubbleRect> = Vec::new();
            for &idx in &initial_config.active_bubble_accounts {
                match commands::bubble::resume_account_bubble_and_chat(&handle, idx, Some(&committed_rects)) {
                    Ok(rect) => {
                        println!("[Startup] Restored bubble {:02} at ({}, {})", idx, rect.x, rect.y);
                        committed_rects.push(rect);
                    }
                    Err(err) => {
                        eprintln!("[Startup ERROR] Failed to restore bubble {}: {}", idx, err);
                    }
                }
            }
            if initial_config.is_dashboard_open {
                println!("[Startup] Dashboard configured ON -> Showing Dashboard");
                let _ = presentation::main_window::get_or_create_main_window(&handle);
            } else {
                println!("[Startup] Dashboard configured OFF -> Hibernating Dashboard");
                presentation::main_window::destroy_or_hibernate_main_window(&handle);
            }
            platforms::messenger::windows::setup_chat_window(&handle, Arc::clone(&chat_state_clone));
            platforms::messenger::daemons::start_auth_watcher(handle.clone(), Arc::clone(&chat_state_clone));
            platforms::messenger::daemons::start_hover_and_autoclose_monitor(handle.clone(), Arc::clone(&chat_state_clone));
            utils::trim_controller::start_idle_trim_sweeper(handle.clone());
            utils::update_notifier::start_background_update_notifier(handle.clone());
            for sc in ["Ctrl+Shift+M", "CommandOrControl+Shift+M"] {
                if let Ok(shortcut) = sc.parse::<Shortcut>() {
                    let _ = app.global_shortcut().register(shortcut);
                }
            }
            let _ = tray::update_tray_menu(&handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { code, api, .. } => {
                let is_explicit = app_handle
                    .try_state::<Arc<ChatState>>()
                    .map(|s| s.is_explicit_exit.load(Ordering::SeqCst))
                    .unwrap_or(false);
                if !is_explicit {
                    println!("[Tauri Lifecycle] Preventing exit: Keeping host & tray running in background (Exit code: {:?})", code);
                    api.prevent_exit();
                } else {
                    println!("[Tauri Lifecycle] User confirmed explicit app exit (Exit code: {:?})", code);
                    std::process::exit(code.unwrap_or(0));
                }
            }
            tauri::RunEvent::Exit => {
                std::process::exit(0);
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                let _ = presentation::main_window::get_or_create_main_window(app_handle);
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.unminimize();
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
            }
            _ => {}
        }
    });
}
