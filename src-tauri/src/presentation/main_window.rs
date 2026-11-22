use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Trims working set memory of current process
pub fn trim_current_process_working_set() {
    crate::operating_systems::current::MemoryOptimizer::trim_current_process_working_set();
}

/// Checks if Win32 HWND is valid in OS
#[cfg(target_os = "windows")]
fn is_window_valid(window: &WebviewWindow) -> bool {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn IsWindow(hWnd: *mut std::ffi::c_void) -> i32;
    }
    if let Ok(hwnd) = window.hwnd() {
        return unsafe { IsWindow(hwnd.0) != 0 };
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn is_window_valid(_window: &WebviewWindow) -> bool {
    true
}

/// Relocates all active bubbles to new screen when Dashboard crosses monitors
pub fn relocate_bubbles_to_screen(
    app: &AppHandle,
    old_screen: crate::movement::ScreenMetrics,
    new_screen: crate::movement::ScreenMetrics,
) {
    let config = crate::movement::get_bubble_physics_config(app);
    let old_bubble_w = (config.size * old_screen.scale_factor).round() as i32;
    let bubble_w = (config.size * new_screen.scale_factor).round() as i32;
    let margin = (config.margin * new_screen.scale_factor).round() as i32;
    let dpi_changed = (new_screen.scale_factor - old_screen.scale_factor).abs() > 0.001;
    let mut relocated: Vec<crate::movement::BubbleRect> = Vec::new();
    for idx in 1..=crate::state::MAX_ACCOUNTS {
        {
            if crate::operating_systems::current::bubble::has_native_bubble(idx)
                && let Some((cur_x, cur_y, _, _)) = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(idx) {
                    if dpi_changed {
                        crate::operating_systems::current::bubble::set_native_bubble_size(idx, bubble_w as u32);
                    }
                    let rel_y_ratio = if old_screen.height > 0 {
                        (cur_y - old_screen.y) as f64 / old_screen.height as f64
                    } else {
                        0.3
                    };
                    let mapped_y = new_screen.y + (rel_y_ratio * new_screen.height as f64) as i32;
                    let (target_x, target_y) = if config.is_free_position {
                        let rel_x_ratio = if old_screen.width > 0 {
                            (cur_x - old_screen.x) as f64 / old_screen.width as f64
                        } else {
                            0.05
                        };
                        let mapped_x = new_screen.x + (rel_x_ratio * new_screen.width as f64) as i32;
                        crate::movement::CollisionResolver::resolve_2d_free_collision(
                            idx,
                            mapped_x,
                            mapped_y,
                            &new_screen,
                            &config,
                            &relocated,
                        )
                    } else {
                        let is_left = cur_x + (old_bubble_w / 2) < old_screen.x + (old_screen.width / 2);
                        let min_y = new_screen.y + margin;
                        let taskbar_margin = (config.taskbar_margin * new_screen.scale_factor).round() as i32;
                        let max_y = (new_screen.y + new_screen.height - taskbar_margin - bubble_w).max(min_y);
                        let clamped_y = mapped_y.clamp(min_y, max_y);
                        let ratio = if crate::operating_systems::current::bubble::is_cursor_over_native_bubble(idx) {
                            config.appear_ratio
                        } else {
                            config.tuck_ratio
                        };
                        let probe_x = if is_left { new_screen.x } else { new_screen.x + new_screen.width };
                        let edge_x = crate::movement::BubbleMovementEngine::calculate_edge_x(
                            &new_screen,
                            &config,
                            probe_x,
                            ratio,
                        );
                        let slots = crate::movement::CollisionResolver::resolve_mutual_column_layout(
                            idx,
                            clamped_y,
                            is_left,
                            &new_screen,
                            &config,
                            &relocated,
                        );
                        let resolved_y = slots.iter().find(|s| s.account_index == idx).map(|s| s.target_y).unwrap_or(clamped_y);
                        (edge_x, resolved_y)
                    };
                    crate::operating_systems::current::bubble::NativeBubbleManager::global().dispatch(
                        crate::operating_systems::current::bubble::NativeBubbleAction::AnimateTo {
                            account_index: idx,
                            target_x,
                            target_y,
                            duration_ms: 200,
                        },
                    );
                    crate::operating_systems::current::bubble::manager::update_cached_bubble_bounds(
                        idx,
                        target_x,
                        target_y,
                        bubble_w as u32,
                        bubble_w as u32,
                    );
                    let rect = crate::movement::BubbleRect {
                        account_index: idx,
                        x: target_x,
                        y: target_y,
                        width: bubble_w,
                        height: bubble_w,
                    };
                    relocated.push(rect);
                    if let Some(cs) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
                        cs.set_bubble_rect(idx, rect);
                    }
                }
        }
        let label = crate::presentation::bubble::get_bubble_window_label(idx);
        if let Some(bubble_win) = app.get_webview_window(&label)
            && bubble_win.is_visible().unwrap_or(false) {
                let _ = crate::commands::bubble::snap_bubble(app.clone(), Some(idx));
            }
    }
}

/// Destroys or hibernates main dashboard window to reach 0 MB UI RAM
pub fn destroy_or_hibernate_main_window(app: &AppHandle) {
    let mut should_notify = false;
    let mut locale = "en".to_string();
    if let Some(state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>()
        && !state.is_explicit_exit.load(std::sync::atomic::Ordering::SeqCst) {
            state.update_user_config(app, |c| c.is_dashboard_open = false);
            should_notify = state.mark_tray_notification_shown();
            locale = state.get_user_config().current_locale().to_string();
        }
    if let Some(main_window) = app.get_webview_window("main") {
        if let Ok(Some(mon)) = main_window.current_monitor() {
            crate::movement::update_target_dashboard_screen(crate::movement::ScreenMetrics::from_monitor(&mon));
        }
        println!("\x1b[35m[MainWindow] Activating Dynamic Destruction: closing main window to 0 MB RAM...\x1b[0m");
        let _ = main_window.hide();
        let _ = main_window.destroy();
        trim_current_process_working_set();
        let _ = crate::tray::update_tray_menu(app);
    }
    if should_notify {
        use tauri_plugin_notification::NotificationExt;
        let (title, body) = if locale == "vi" {
            ("DesktopBubbleChat đang chạy ngầm", "Ứng dụng vẫn đang nhận tin nhắn trong khay hệ thống. Bấm vào biểu tượng hoặc phím tắt (Ctrl+Shift+M / ⌘⇧M) để mở lại.")
        } else {
            ("DesktopBubbleChat is running in background", "The app is still receiving messages in system tray. Click the tray icon or use shortcut (Ctrl+Shift+M / ⌘⇧M) to reopen.")
        };
        let _ = app.notification().builder().title(title).body(body).show();
    }
}

/// Registers WindowEvent handlers for main window
pub fn setup_main_window_events(main_window: &WebviewWindow, app: &AppHandle) {
    let app_handle = app.clone();
    let win = main_window.clone();
    main_window.on_window_event(move |event| {
        match event {
            tauri::WindowEvent::Moved(_) => {
                if let Ok(Some(mon)) = win.current_monitor() {
                    let new_metrics = crate::movement::ScreenMetrics::from_monitor(&mon);
                    if let Some((old_screen, target_screen)) = crate::movement::update_target_dashboard_screen(new_metrics) {
                        relocate_bubbles_to_screen(&app_handle, old_screen, target_screen);
                    }
                }
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let is_explicit = app_handle
                    .try_state::<std::sync::Arc<crate::state::ChatState>>()
                    .map(|s| s.is_explicit_exit.load(std::sync::atomic::Ordering::SeqCst))
                    .unwrap_or(false);
                if !is_explicit {
                    api.prevent_close();
                    use tauri::Emitter;
                    let _ = app_handle.emit("show-close-dialog", ());
                }
            }
            _ => {}
        }
    });
}

/// Creates or restores main dashboard window
pub fn get_or_create_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
        state.update_user_config(app, |c| c.is_dashboard_open = true);
    }
    if let Some(existing) = app.get_webview_window("main") {
        if is_window_valid(&existing) {
            if let Ok(Some(mon)) = existing.current_monitor() {
                let new_metrics = crate::movement::ScreenMetrics::from_monitor(&mon);
                if let Some((old_screen, target_screen)) = crate::movement::update_target_dashboard_screen(new_metrics) {
                    relocate_bubbles_to_screen(app, old_screen, target_screen);
                }
            }
            let _ = existing.show();
            let _ = existing.unminimize();
            let _ = existing.set_focus();
            let _ = crate::tray::update_tray_menu(app);
            return Ok(existing);
        } else {
            let _ = existing.destroy();
        }
    }
    println!("\x1b[32m[MainWindow] Recreating Dashboard window from local bundle...\x1b[0m");
    let new_window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("DesktopBubbleChat")
        .inner_size(1200.0, 800.0)
        .min_inner_size(980.0, 640.0)
        .resizable(true)
        .decorations(false)
        .transparent(false)
        .center()
        .additional_browser_args("--renderer-process-limit=1 --disable-background-networking")
        .build()
        .map_err(|e| format!("Failed to recreate Dashboard window: {}", e))?;
    setup_main_window_events(&new_window, app);
    if let Ok(Some(mon)) = new_window.current_monitor() {
        let new_metrics = crate::movement::ScreenMetrics::from_monitor(&mon);
        if let Some((old_screen, target_screen)) = crate::movement::update_target_dashboard_screen(new_metrics) {
            relocate_bubbles_to_screen(app, old_screen, target_screen);
        }
    }
    let _ = new_window.show();
    let _ = new_window.set_focus();
    let _ = crate::tray::update_tray_menu(app);
    Ok(new_window)
}

/// Sets up main window event handlers on initial app startup
pub fn setup_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        if let Ok(Some(mon)) = main_window.current_monitor() {
            let new_metrics = crate::movement::ScreenMetrics::from_monitor(&mon);
            let _ = crate::movement::update_target_dashboard_screen(new_metrics);
        }
        setup_main_window_events(&main_window, app);
    }
}
