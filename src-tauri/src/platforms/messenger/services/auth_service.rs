use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::platforms::messenger::constants::{MESSENGER_BASE_URL, MESSENGER_INJECT_SCRIPT};
use crate::platforms::messenger::verifier::is_window_logged_in;
use crate::platforms::messenger::windows::{
    apply_window_theme_color, get_account_theme, get_messenger_window_label,
    get_or_create_messenger_window, hide_all_other_messenger_windows,
};
use crate::state::ChatState;
use crate::windows::bubble::{ensure_bubble_positioned_and_visible, get_bubble_window_label};

pub struct AuthService;

impl AuthService {
    /// Mở cửa sổ popup đăng nhập Messenger độc lập cho tài khoản chỉ định
    pub async fn open_auth(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        println!("\x1b[32m[BE - Auth] ========================================================\x1b[0m");
        println!("\x1b[32m[BE - Auth] >>> NHẬN LỆNH MỞ AUTH POPUP: account_index = {} \x1b[0m", idx);

        // 0. Xóa trạng thái auth cũ của tài khoản này để bắt đầu phiên xác thực sạch sẽ
        state.clear_account_auth(idx);

        // 1. Lấy hoặc tạo cửa sổ chat Messenger tương ứng với profile riêng biệt
        let chat_window = match get_or_create_messenger_window(app, idx, Arc::clone(state)) {
            Ok(win) => {
                println!("[BE - Auth] Lấy/tạo thành công cửa sổ: '{}'", win.label());
                win
            }
            Err(err) => {
                eprintln!("\x1b[31m[BE - Auth ERROR] get_or_create_messenger_window thất bại: {}\x1b[0m", err);
                return Err(err);
            }
        };

        // 2. Ẩn tất cả các cửa sổ chat của tài khoản khác để tránh đè giao diện
        hide_all_other_messenger_windows(app, &chat_window.label());

        // 3. Bật Auth Mode riêng cho cửa sổ này (chặn tự động ẩn khi blur)
        state.set_window_auth_mode(&chat_window.label(), true);

        // 4. Thiết lập giao diện Popup đăng nhập độc lập rõ ràng (có tiêu đề và nút đóng X)
        let theme = get_account_theme(idx);
        let _ = chat_window.set_title(&format!("Đăng nhập Messenger {:02}", idx));
        let _ = chat_window.set_decorations(true);
        let _ = chat_window.set_skip_taskbar(false);
        let _ = chat_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 560.0,
            height: 720.0,
        }));
        let _ = chat_window.center();
        let _ = chat_window.unminimize();
        let _ = chat_window.set_always_on_top(true);

        #[cfg(target_os = "windows")]
        apply_window_theme_color(&chat_window, theme.hex, theme.border_hex);

        // Đảm bảo URL đang ở trang Messenger nếu cửa sổ đang trống hoặc chưa ở trang messenger/2FA
        if let Ok(url) = tauri::Url::parse(MESSENGER_BASE_URL) {
            let current_url = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
            println!("[BE - Auth] URL hiện tại của cửa sổ: '{}'", current_url);
            let in_2fa = current_url.contains("checkpoint") || current_url.contains("two_step_verification");
            let in_messenger = current_url.contains("messenger.com");
            if current_url.is_empty() || current_url == "about:blank" || (!in_2fa && !in_messenger) {
                println!("[BE - Auth] Điều hướng sang: {}", MESSENGER_BASE_URL);
                let _ = chat_window.navigate(url);
            }
        }

        // 5. Nạp injected script với account_index
        let init_code = format!(
            "window.__ACCOUNT_INDEX__ = {};\n{}",
            idx,
            MESSENGER_INJECT_SCRIPT
        );
        let _ = chat_window.eval(&init_code);

        // 6. Hiển thị và đưa popup lên tiêu điểm phía trước người dùng
        println!("[BE - Auth] Gọi chat_window.show()...");
        if let Err(e) = chat_window.show() {
            eprintln!("\x1b[31m[BE - Auth ERROR] chat_window.show() lỗi: {:?}\x1b[0m", e);
            return Err(e.to_string());
        }
        println!("[BE - Auth] Gọi chat_window.set_focus()...");
        if let Err(e) = chat_window.set_focus() {
            eprintln!("\x1b[31m[BE - Auth ERROR] chat_window.set_focus() lỗi: {:?}\x1b[0m", e);
        }

        let _ = app.emit("chat-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": true,
            "windowLabel": chat_window.label()
        }));
        println!("\x1b[32m[BE - Auth] HOÀN TẤT: Cửa sổ Đăng nhập Messenger {:02} đã được show() và set_focus()!\x1b[0m", idx);
        println!("\x1b[32m[BE - Auth] ========================================================\x1b[0m");

        Ok(())
    }

    /// Ngắt kết nối Messenger, điều hướng về login và ẩn cửa sổ
    pub fn disconnect(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!("[BE - Messenger] Ngắt kết nối tài khoản Messenger {:02}", idx);
        state.set_window_auth_mode(&label, false);
        state.clear_account_auth(idx);
        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

        if let Some(chat_window) = win {
            let nav_cmd = format!("window.location.href = '{}';", MESSENGER_BASE_URL);
            let _ = chat_window.eval(&nav_cmd);
            let _ = chat_window.hide();
            let _ = chat_window.set_decorations(false);
        }

        // Ẩn bubble tương ứng nếu có
        let bubble_label = get_bubble_window_label(idx);
        if let Some(bubble_win) = app.get_webview_window(&bubble_label) {
            let _ = bubble_win.hide();
        }

        let _ = app.emit("messenger-auth-status", serde_json::json!({
            "accountIndex": idx,
            "connected": false
        }));

        Ok(())
    }

    /// Kiểm tra trạng thái đăng nhập thực tế của tài khoản Messenger
    pub fn check_auth(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<serde_json::Value, String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);

        // 1. Kiểm tra trạng thái đã được xác nhận trong state SSOT
        if let Some(auth_info) = state.get_account_auth(idx) {
            if auth_info.connected {
                return Ok(serde_json::json!({
                    "accountIndex": idx,
                    "connected": true,
                    "name": auth_info.name,
                    "userId": auth_info.user_id,
                }));
            }
        }

        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

        if let Some(chat_window) = win {
            // Kích hoạt re-check nhanh trong webview
            let trigger_script = format!(
                "window.__ACCOUNT_INDEX__ = {}; if (typeof window.checkAndReportAuth === 'function') {{ window.checkAndReportAuth(); }}",
                idx
            );
            let _ = chat_window.eval(&trigger_script);

            let (is_logged_in, user_name) = is_window_logged_in(&chat_window);
            let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
            let title = chat_window.title().unwrap_or_default();

            if is_logged_in {
                println!("\x1b[32m[BE - Auth] check_messenger_auth: XÁC THỰC THÀNH CÔNG cho tài khoản {:02}! (User: '{}', URL: {})\x1b[0m", idx, user_name, url_str);
                state.set_account_auth(idx, true, if user_name.is_empty() { None } else { Some(user_name.clone()) });
                state.set_window_auth_mode(&label, false);
                let _ = chat_window.set_title("Messenger");
                let _ = chat_window.hide();
                let _ = chat_window.set_decorations(false);
                let _ = chat_window.set_skip_taskbar(true);
                let _ = ensure_bubble_positioned_and_visible(app, idx);
                let _ = app.emit("chat-visibility-changed", serde_json::json!({
                    "accountIndex": idx,
                    "visible": false,
                    "windowLabel": &label
                }));
                let _ = app.emit("messenger-auth-status", serde_json::json!({
                    "accountIndex": idx,
                    "connected": true,
                    "name": &user_name
                }));
            }

            return Ok(serde_json::json!({
                "accountIndex": idx,
                "connected": is_logged_in,
                "name": if is_logged_in { user_name } else { "".to_string() },
                "url": url_str,
                "title": title
            }));
        }
        // Cửa sổ chưa tồn tại: trả về false im lặng, không spam terminal
        Ok(serde_json::json!({ "accountIndex": idx, "connected": false, "name": "" }))
    }

    /// Nhận báo cáo trạng thái xác thực từ webview và đồng bộ sang UI
    pub fn report_status(
        app: &AppHandle,
        state: &Arc<ChatState>,
        connected: bool,
        name: Option<String>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);

        println!(
            "\x1b[32m[BE - Auth] report_auth_status NHẬN TỪ WEBVIEW: account={:02}, connected={}, user={:?}\x1b[0m",
            idx, connected, name
        );

        state.set_account_auth(idx, connected, name.clone());

        if connected {
            state.set_window_auth_mode(&label, false);
            if let Some(win) = app.get_webview_window(&label) {
                // Nếu cửa sổ đang ở domain facebook.com, lập tức chuyển hướng về messenger.com
                if let Ok(curr_url) = win.url() {
                    let u = curr_url.to_string();
                    if u.contains("facebook.com") {
                        println!("[BE - Auth] report_auth_status: Cửa sổ đang ở Facebook ('{}'), điều hướng về https://www.messenger.com/", u);
                        if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                            let _ = win.navigate(target);
                        }
                    }
                }
                let _ = win.set_title("Messenger");
                let _ = win.hide();
                let _ = win.set_decorations(false);
                let _ = win.set_skip_taskbar(true);
            }
            let _ = ensure_bubble_positioned_and_visible(app, idx);
            let _ = app.emit("chat-visibility-changed", serde_json::json!({
                "accountIndex": idx,
                "visible": false,
                "windowLabel": &label
            }));
        }

        let _ = app.emit("messenger-auth-status", serde_json::json!({
            "accountIndex": idx,
            "connected": connected,
            "name": name
        }));
        Ok(())
    }
}
