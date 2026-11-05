use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Trims working set memory of current process to standby/paging list
#[cfg(target_os = "windows")]
pub fn trim_current_process_working_set() {
    unsafe {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn SetProcessWorkingSetSize(
                hProcess: *mut std::ffi::c_void,
                dwMinimumWorkingSetSize: usize,
                dwMaximumWorkingSetSize: usize,
            ) -> i32;
        }
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

/// Checks if Win32 HWND is valid in OS
#[cfg(target_os = "windows")]
fn is_window_valid(window: &WebviewWindow) -> bool {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn IsWindow(hWnd: *mut std::ffi::c_void) -> i32;
    }
    if let Ok(hwnd) = window.hwnd() {
        return unsafe { IsWindow(hwnd.0 as *mut std::ffi::c_void) != 0 };
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn is_window_valid(_window: &WebviewWindow) -> bool {
    true
}

/// Destroys or hibernates main dashboard window to reach 0 MB UI RAM
pub fn destroy_or_hibernate_main_window(app: &AppHandle) {
    if let Some(state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
        state.update_user_config(app, |c| c.is_dashboard_open = false);
    }
    if let Some(main_window) = app.get_webview_window("main") {
        println!("\x1b[35m[MainWindow] Activating Dynamic Destruction: closing main window to 0 MB RAM...\x1b[0m");
        let _ = main_window.hide();
        let _ = main_window.destroy();
        #[cfg(target_os = "windows")]
        trim_current_process_working_set();
        let _ = crate::tray::update_tray_menu(app);
    }
}

/// Registers CloseRequested handler for main window
pub fn setup_main_window_events(main_window: &WebviewWindow, app: &AppHandle) {
    let app_handle = app.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            destroy_or_hibernate_main_window(&app_handle);
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
    let _ = new_window.show();
    let _ = new_window.set_focus();
    let _ = crate::tray::update_tray_menu(app);
    Ok(new_window)
}

/// Sets up main window event handlers on initial app startup
pub fn setup_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        setup_main_window_events(&main_window, app);
    }
}
