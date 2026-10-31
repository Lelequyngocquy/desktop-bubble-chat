use tauri::{AppHandle, Manager};

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

/// Cấu hình sự kiện cho cửa sổ chính (Home Page): Chặn thoát app khi ấn nút X, ẩn về tray
pub fn setup_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        let main_clone = main_window.clone();
        main_window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = main_clone.hide();
                #[cfg(target_os = "windows")]
                trim_current_process_working_set();
            }
        });
    }
}
