use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use super::theme::{apply_window_theme_color, get_account_theme};
use crate::platforms::messenger::constants::{MESSENGER_INJECT_SCRIPT, MESSENGER_LOGIN_URL};
use crate::platforms::messenger::services::ChatService;
use crate::state::ChatState;
use crate::utils::time::now_millis;
use crate::windows::bubble::tuck_bubble;

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
        let idx = win_label
            .strip_prefix("chat-messenger-")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        match event {
            tauri::WindowEvent::Focused(false) => {
                // Chỉ tự động ẩn khi KHÔNG ở chế độ đăng nhập (Auth Mode) của đúng cửa sổ này (SSOT)
                if !state_clone.is_window_in_auth_mode(&win_label) {
                    state_clone.last_blur_time.store(now_millis(), Ordering::SeqCst);
                    let _ = ChatService::hide_chat(&app_handle, idx);
                }
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // Chặn đóng tiến trình, ẩn cửa sổ và reset chế độ Auth cho riêng cửa sổ này
                api.prevent_close();
                state_clone.set_window_auth_mode(&win_label, false);
                let _ = chat_clone.set_decorations(false);
                let _ = chat_clone.set_skip_taskbar(true);
                let _ = app_handle.emit("messenger-auth-closed", serde_json::json!({ "label": &win_label }));
                let _ = ChatService::hide_chat(&app_handle, idx);
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

    let parsed_url = tauri::Url::parse(MESSENGER_LOGIN_URL).map_err(|e| e.to_string())?;

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
/// và tất cả các bubble khác đều bị thụt vào mép (tuck)
pub fn hide_all_other_messenger_windows(app: &AppHandle, active_label: &str) {
    for idx in 1..=5 {
        let label = get_messenger_window_label(idx);
        if label != active_label {
            if let Some(w) = app.get_webview_window(&label) {
                let _ = w.hide();
            }
            let _ = app.emit("chat-visibility-changed", serde_json::json!({
                "accountIndex": idx,
                "visible": false,
                "windowLabel": &label
            }));
            tuck_bubble(app, Some(idx));
        }
    }
    if active_label != "chat-messenger" {
        if let Some(w) = app.get_webview_window("chat-messenger") {
            let _ = w.hide();
            let _ = app.emit("chat-visibility-changed", serde_json::json!({
                "accountIndex": 1,
                "visible": false,
                "windowLabel": "chat-messenger"
            }));
            tuck_bubble(app, Some(1));
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
