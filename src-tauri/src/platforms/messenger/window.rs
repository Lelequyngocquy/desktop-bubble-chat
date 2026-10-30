use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::ChatState;
use crate::utils::positioning::{is_cursor_over_any_bubble, tuck_bubble};
use crate::utils::time::now_millis;

pub const MESSENGER_INJECT_SCRIPT: &str = include_str!("scripts/messenger_inject.js");

#[derive(Clone, Copy, Debug)]
pub struct AccountTheme {
    pub hex: &'static str,
    pub border_hex: &'static str,
}

pub fn get_account_theme(idx: u32) -> AccountTheme {
    match idx {
        1 => AccountTheme {
            hex: "#06B6D4",
            border_hex: "#0891B2",
        },
        2 => AccountTheme {
            hex: "#EC4899",
            border_hex: "#DB2777",
        },
        3 => AccountTheme {
            hex: "#F97316",
            border_hex: "#EA580C",
        },
        4 => AccountTheme {
            hex: "#8B5CF6",
            border_hex: "#7C3AED",
        },
        5 => AccountTheme {
            hex: "#10B981",
            border_hex: "#059669",
        },
        _ => AccountTheme {
            hex: "#06B6D4",
            border_hex: "#0891B2",
        },
    }
}

pub fn hex_to_colorref(hex: &str) -> u32 {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u32::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u32::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u32::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        r | (g << 8) | (b << 16)
    } else {
        0
    }
}

#[cfg(target_os = "windows")]
pub fn apply_window_theme_color(window: &tauri::WebviewWindow, hex: &str, border_hex: &str) {
    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: *mut std::ffi::c_void,
            dwAttribute: u32,
            pvAttribute: *const std::ffi::c_void,
            cbAttribute: u32,
        ) -> i32;
    }

    if let Ok(hwnd) = window.hwnd() {
        let raw_hwnd = hwnd.0 as *mut std::ffi::c_void;
        let caption_color = hex_to_colorref(hex);
        let border_color = hex_to_colorref(border_hex);
        let text_color: u32 = 0x00FFFFFF; // Màu chữ trắng

        const DWMWA_BORDER_COLOR: u32 = 34;
        const DWMWA_CAPTION_COLOR: u32 = 35;
        const DWMWA_TEXT_COLOR: u32 = 36;

        unsafe {
            let _ = DwmSetWindowAttribute(
                raw_hwnd,
                DWMWA_CAPTION_COLOR,
                &caption_color as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
            let _ = DwmSetWindowAttribute(
                raw_hwnd,
                DWMWA_TEXT_COLOR,
                &text_color as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
            let _ = DwmSetWindowAttribute(
                raw_hwnd,
                DWMWA_BORDER_COLOR,
                &border_color as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

/// Trả về label định danh cửa sổ chat Messenger theo số thứ tự tài khoản (ví dụ: chat-messenger-01)
pub fn get_messenger_window_label(account_index: u32) -> String {
    format!("chat-messenger-{:02}", account_index)
}

/// Gắn các listener xử lý sự kiện Focus/Blur và chặn đóng cho cửa sổ chat
pub fn setup_window_events(app: &AppHandle, chat_window: &tauri::WebviewWindow, chat_state: Arc<ChatState>) {
    let chat_clone = chat_window.clone();
    let app_handle = app.clone();
    let state_clone = Arc::clone(&chat_state);
    let win_label = chat_window.label().to_string();

    chat_window.on_window_event(move |event| {
        match event {
            tauri::WindowEvent::Focused(false) => {
                // Chỉ tự động ẩn khi KHÔNG ở chế độ đăng nhập (Auth Mode) của đúng cửa sổ này (SSOT)
                if !state_clone.is_window_in_auth_mode(&win_label) {
                    state_clone.last_blur_time.store(now_millis(), Ordering::SeqCst);
                    let _ = chat_clone.hide();
                    let _ = app_handle.emit("chat-visibility-changed", false);

                    // Khi click ra ngoài cả popup lẫn bubble -> thụt tất cả bubble vào mép
                    if !is_cursor_over_any_bubble(&app_handle) {
                        tuck_bubble(&app_handle, None);
                    }
                }
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // Chặn đóng tiến trình, ẩn cửa sổ và reset chế độ Auth cho riêng cửa sổ này
                api.prevent_close();
                state_clone.set_window_auth_mode(&win_label, false);
                let _ = chat_clone.hide();
                let _ = chat_clone.set_decorations(false);
                let _ = chat_clone.set_skip_taskbar(true);
                let _ = app_handle.emit("chat-visibility-changed", false);
                let _ = app_handle.emit("messenger-auth-closed", serde_json::json!({ "label": &win_label }));

                if !is_cursor_over_any_bubble(&app_handle) {
                    tuck_bubble(&app_handle, None);
                }
            }
            _ => {}
        }
    });
}

/// Tìm hoặc khởi tạo động WebviewWindow với profile data_directory riêng biệt cho từng tài khoản
pub fn get_or_create_messenger_window(
    app: &AppHandle,
    account_index: u32,
    chat_state: Arc<ChatState>,
) -> Result<tauri::WebviewWindow, String> {
    let label = get_messenger_window_label(account_index);
    println!("[BE - Window] get_or_create_messenger_window cho label='{}' (index={})", label, account_index);

    // 1. Kiểm tra cửa sổ đã tồn tại theo label tuần tự
    if let Some(win) = app.get_webview_window(&label) {
        println!("[BE - Window] Cửa sổ '{}' đã tồn tại, tái sử dụng.", label);
        return Ok(win);
    }

    // 2. Tương thích ngược: Với tài khoản 01, nếu có sẵn cửa sổ 'chat-messenger' từ config
    if account_index == 1 {
        if let Some(win) = app.get_webview_window("chat-messenger") {
            println!("[BE - Window] Tìm thấy cửa sổ legacy 'chat-messenger' cho tài khoản 1.");
            return Ok(win);
        }
    }

    // 3. Khởi tạo WebView2 động với User Data Folder (data_directory) hoàn toàn riêng biệt
    let mut profile_dir = app.path().app_data_dir().map_err(|e| {
        let msg = format!("Không lấy được app_data_dir: {}", e);
        eprintln!("[BE - Window ERROR] {}", msg);
        msg
    })?;
    profile_dir.push("profiles");
    profile_dir.push(format!("messenger_{:02}", account_index));
    let _ = std::fs::create_dir_all(&profile_dir);
    println!("[BE - Window] Thư mục profile: {:?}", profile_dir);

    let parsed_url = tauri::Url::parse("https://www.messenger.com/login/").map_err(|e| e.to_string())?;

    let theme = get_account_theme(account_index);
    let init_script = format!(
        "window.__ACCOUNT_INDEX__ = {};\n{}",
        account_index,
        MESSENGER_INJECT_SCRIPT
    );

    println!("[BE - Window] Đang build WebviewWindow mới cho '{}'...", label);
    let builder = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::External(parsed_url),
    )
    .data_directory(profile_dir)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0")
    .initialization_script(&init_script)
    .title(format!("Đăng nhập Messenger {:02}", account_index))
    .inner_size(560.0, 720.0)
    .min_inner_size(420.0, 560.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(true)
    .skip_taskbar(false)
    .shadow(true)
    .visible(false);

    match builder.build() {
        Ok(window) => {
            println!("\x1b[32m[BE - Window] Build thành công cửa sổ Webview: {}\x1b[0m", label);
            setup_window_events(app, &window, chat_state);
            #[cfg(target_os = "windows")]
            apply_window_theme_color(&window, theme.hex, theme.border_hex);
            Ok(window)
        }
        Err(err) => {
            let msg = format!("Builder.build() thất bại cho '{}': {:?}", label, err);
            eprintln!("\x1b[31m[BE - Window ERROR] {}\x1b[0m", msg);
            Err(msg)
        }
    }
}

/// Đảm bảo chỉ có 1 cửa sổ chat/auth Messenger được hiển thị tại một thời điểm
pub fn hide_all_other_messenger_windows(app: &AppHandle, active_label: &str) {
    for idx in 1..=5 {
        let label = get_messenger_window_label(idx);
        if label != active_label {
            if let Some(w) = app.get_webview_window(&label) {
                let _ = w.hide();
            }
        }
    }
    if active_label != "chat-messenger" {
        if let Some(w) = app.get_webview_window("chat-messenger") {
            let _ = w.hide();
        }
    }
}

/// Cấu hình sự kiện tự động ẩn khi blur và chặn close cho cửa sổ chat Messenger mặc định
pub fn setup_chat_window(app: &AppHandle, chat_state: Arc<ChatState>) {
    if let Some(chat_window) = app.get_webview_window("chat-messenger") {
        setup_window_events(app, &chat_window, chat_state);
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);
    }
}

/// Kiểm tra trạng thái đăng nhập chuẩn theo đặc tả c_user và Routing Invariants (standard_mess_login.md)
/// Hàm thuần túy truy vấn (Read-Only), tuyệt đối không gây side-effect điều hướng webview trong quá trình polling.
pub fn is_window_logged_in(chat_window: &tauri::WebviewWindow) -> (bool, String) {
    let title = chat_window.title().unwrap_or_default();
    let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();

    // 1. Nếu đang ở domain facebook.com hoặc meta.com:
    // Đang trong luồng xác thực / đăng nhập / 2FA / checkpoint của Meta -> Chưa hoàn tất đăng nhập Messenger
    if url_str.contains("facebook.com") || url_str.contains("meta.com") {
        return (false, String::new());
    }

    // 2. Kênh tốc độ cao: Đọc UID trực tiếp từ hash URL do injected script đóng dấu (#auth_uid=...)
    if let Some(pos) = url_str.find("#auth_uid=") {
        let uid_part = &url_str[pos + 10..];
        let clean_uid = uid_part.split(['&', '#', '/']).next().unwrap_or("").trim().to_string();
        if !clean_uid.is_empty() {
            return (true, clean_uid);
        }
    }

    // 3. Thẻ xác thực chuẩn [AUTH_OK] được injected script đóng dấu khi tìm thấy cookie c_user trên messenger.com
    if title.contains("[AUTH_OK]") {
        return (true, String::new());
    }

    // 4. Bất biến Routing: Trên domain messenger.com, CHỈ KHI vào hẳn thread chat (/t/ hoặc /e2ee/)
    // mới coi là đã đăng nhập thành công.
    // LƯU Ý QUAN TRỌNG: Tuyệt đối KHÔNG coi https://www.messenger.com/ (không có /login) là logged in,
    // vì Meta thường xuyên render form đăng nhập trực tiếp trên root path "/"!
    if url_str.contains("messenger.com") && (url_str.contains("/t/") || url_str.contains("/e2ee/")) {
        return (true, String::new());
    }

    (false, String::new())
}
