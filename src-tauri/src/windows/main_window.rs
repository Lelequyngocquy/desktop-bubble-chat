use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Thu hồi bộ nhớ vật lý (Working Set) của tiến trình hiện tại về standby/paging list
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

/// Kiểm tra HWND Win32 có còn tồn tại và hợp lệ trong hệ điều hành hay không
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

/// Tiêu hủy hoặc ngủ đông cửa sổ chính (Home Dashboard) để đưa RAM UI về 0 MB
pub fn destroy_or_hibernate_main_window(app: &AppHandle) {
    if let Some(state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
        state.update_user_config(app, |c| c.is_dashboard_open = false);
    }

    if let Some(main_window) = app.get_webview_window("main") {
        println!("\x1b[35m[MainWindow] Kích hoạt Dynamic Destruction: Đóng và tiêu hủy cửa sổ chính về 0 MB RAM...\x1b[0m");
        let _ = main_window.hide();
        let _ = main_window.destroy();
        #[cfg(target_os = "windows")]
        trim_current_process_working_set();
        let _ = crate::tray::update_tray_menu(app);
    }
}

/// Đăng ký bắt sự kiện CloseRequested cho cửa sổ chính (chặn thoát, kích hoạt Dynamic Destruction)
pub fn setup_main_window_events(main_window: &WebviewWindow, app: &AppHandle) {
    let app_handle = app.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            destroy_or_hibernate_main_window(&app_handle);
        }
    });
}

/// Khởi tạo hoặc lấy lại cửa sổ chính (Home Dashboard)
pub fn get_or_create_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
        state.update_user_config(app, |c| c.is_dashboard_open = true);
    }

    // 1. Kiểm tra cửa sổ đã tồn tại và HWND còn sống hay không
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

    // 2. Tái tạo cửa sổ mới từ bundle Vite cục bộ (index.html)
    println!("\x1b[32m[MainWindow] Tái sinh cửa sổ Dashboard chính từ local bundle...\x1b[0m");
    let new_window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("DesktopBubbleChat - Bong Bóng Trò Chuyện")
        .inner_size(1200.0, 800.0)
        .min_inner_size(980.0, 640.0)
        .resizable(true)
        .decorations(false)
        .transparent(false)
        .center()
        .additional_browser_args("--renderer-process-limit=1 --disable-gpu-rasterization --enable-low-end-device-mode --disable-background-networking")
        .build()
        .map_err(|e| format!("Không thể tái tạo cửa sổ Dashboard: {}", e))?;

    // 3. Gắn lại sự kiện CloseRequested cho instance mới
    setup_main_window_events(&new_window, app);

    let _ = new_window.show();
    let _ = new_window.set_focus();
    let _ = crate::tray::update_tray_menu(app);

    Ok(new_window)
}

/// Cấu hình sự kiện cho cửa sổ chính ban đầu khi app khởi động
pub fn setup_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        setup_main_window_events(&main_window, app);
    }
}
