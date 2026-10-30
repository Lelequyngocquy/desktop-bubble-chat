use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::ChatState;
use crate::utils::positioning::{
    dock_chat_window, ensure_bubble_positioned_and_visible, get_bubble_window_label,
    is_cursor_over_any_bubble, tuck_bubble,
};
use crate::utils::time::now_millis;
use super::parser::parse_unread_from_title;
use super::window::{
    apply_window_theme_color, get_account_theme, get_messenger_window_label,
    get_or_create_messenger_window, hide_all_other_messenger_windows, is_window_logged_in,
    MESSENGER_INJECT_SCRIPT,
};

/// Command mở cửa sổ popup đăng nhập Messenger độc lập cho tài khoản chỉ định
#[tauri::command]
pub async fn open_messenger_auth(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    println!("\x1b[32m[BE - Auth] ========================================================\x1b[0m");
    println!("\x1b[32m[BE - Auth] >>> NHẬN LỆNH MỞ AUTH POPUP: account_index = {:?} (dùng idx: {})\x1b[0m", account_index, idx);

    // 0. Xóa trạng thái auth cũ của tài khoản này để bắt đầu phiên xác thực sạch sẽ
    state.clear_account_auth(idx);

    // 1. Lấy hoặc tạo cửa sổ chat Messenger tương ứng với profile riêng biệt
    let chat_window = match get_or_create_messenger_window(&app, idx, Arc::clone(&state)) {
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
    hide_all_other_messenger_windows(&app, &chat_window.label());

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

    // Đảm bảo URL đang ở trang đăng nhập Messenger nếu cửa sổ đang trống hoặc chưa ở trang login/2FA
    if let Ok(url) = tauri::Url::parse("https://www.messenger.com/login/") {
        let current_url = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
        println!("[BE - Auth] URL hiện tại của cửa sổ: '{}'", current_url);
        let in_2fa = current_url.contains("checkpoint") || current_url.contains("two_step_verification");
        if current_url.is_empty() || current_url == "about:blank" || (!in_2fa && !current_url.contains("/login")) {
            println!("[BE - Auth] Điều hướng sang: https://www.messenger.com/login/");
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

    let _ = app.emit("chat-visibility-changed", true);
    println!("\x1b[32m[BE - Auth] HOÀN TẤT: Cửa sổ Đăng nhập Messenger {:02} đã được show() và set_focus()!\x1b[0m", idx);
    println!("\x1b[32m[BE - Auth] ========================================================\x1b[0m");

    Ok(())
}

/// Command mở trực tiếp khung chat cạnh bong bóng nổi cho tài khoản chỉ định
#[tauri::command]
pub async fn open_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let _ = ensure_bubble_positioned_and_visible(&app, idx);

    let chat_window = get_or_create_messenger_window(&app, idx, Arc::clone(&state))?;

    // Ẩn tất cả các cửa sổ chat của tài khoản khác
    hide_all_other_messenger_windows(&app, &chat_window.label());

    // Đảm bảo URL luôn ở messenger.com, ngăn chặn tuyệt đối việc hiển thị giao diện facebook.com
    if let Ok(curr_url) = chat_window.url() {
        let u = curr_url.to_string();
        if !u.contains("messenger.com") || u.contains("facebook.com") {
            println!("[BE - Chat] Cửa sổ đang ở URL '{}', chuyển hướng bắt buộc về https://www.messenger.com/", u);
            if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                let _ = chat_window.navigate(target);
            }
        }
    }

    state.set_window_auth_mode(&chat_window.label(), false);
    let _ = chat_window.set_decorations(false);
    let _ = chat_window.set_skip_taskbar(true);
    dock_chat_window(&app, &chat_window, idx);
    let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

    if let Ok(title) = chat_window.title() {
        let count = parse_unread_from_title(&title);
        let _ = app.emit("badge-updated", count);
    }

    println!("[BE - Chat] Mở khung chat Messenger {:02} docked cạnh Bubble", idx);
    chat_window.show().map_err(|e| e.to_string())?;
    chat_window.set_focus().map_err(|e| e.to_string())?;
    let _ = app.emit("bubble-visibility-changed", true);
    let _ = app.emit("chat-visibility-changed", true);

    Ok(())
}

/// Command bật/tắt khung chat Messenger khi người dùng click vào bong bóng nổi
#[tauri::command]
pub async fn toggle_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    let chat_window = get_or_create_messenger_window(&app, idx, Arc::clone(&state))?;

    let is_visible = chat_window.is_visible().unwrap_or(false);

    if is_visible {
        state.last_blur_time.store(now_millis(), Ordering::SeqCst);
        chat_window.hide().map_err(|e| e.to_string())?;
        let _ = app.emit("chat-visibility-changed", false);
        if !is_cursor_over_any_bubble(&app) {
            tuck_bubble(&app, Some(idx));
        }
        Ok(false)
    } else {
        let last_blur = state.last_blur_time.load(Ordering::SeqCst);
        if now_millis() - last_blur < 250 {
            return Ok(false);
        }

        // Ẩn các cửa sổ của tài khoản khác trước khi bung mở
        hide_all_other_messenger_windows(&app, &chat_window.label());

        // Đảm bảo URL luôn ở messenger.com, ngăn chặn tuyệt đối việc hiển thị giao diện facebook.com
        if let Ok(curr_url) = chat_window.url() {
            let u = curr_url.to_string();
            if !u.contains("messenger.com") || u.contains("facebook.com") {
                println!("[BE - Chat] Cửa sổ đang ở URL '{}', chuyển hướng bắt buộc về https://www.messenger.com/", u);
                if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                    let _ = chat_window.navigate(target);
                }
            }
        }

        state.set_window_auth_mode(&chat_window.label(), false);
        let _ = chat_window.set_decorations(false);
        let _ = chat_window.set_skip_taskbar(true);
        dock_chat_window(&app, &chat_window, idx);
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

        if let Ok(title) = chat_window.title() {
            let count = parse_unread_from_title(&title);
            let _ = app.emit("badge-updated", count);
        }

        chat_window.show().map_err(|e| e.to_string())?;
        chat_window.set_focus().map_err(|e| e.to_string())?;
        let _ = app.emit("chat-visibility-changed", true);
        Ok(true)
    }
}

/// Command ngắt kết nối Messenger, điều hướng về login và ẩn cửa sổ
#[tauri::command]
pub fn disconnect_messenger(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let label = get_messenger_window_label(idx);
    println!("[BE - Messenger] Ngắt kết nối tài khoản Messenger {:02}", idx);
    state.set_window_auth_mode(&label, false);
    state.clear_account_auth(idx);
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

    if let Some(chat_window) = win {
        let _ = chat_window.eval("window.location.href = 'https://www.messenger.com/login/';");
        let _ = chat_window.hide();
        let _ = chat_window.set_decorations(false);
    }

    // Ẩn bubble tương ứng nếu có
    let bubble_label = get_bubble_window_label(idx);
    if let Some(bubble_win) = app.get_webview_window(&bubble_label).or_else(|| if idx == 1 { app.get_webview_window("bubble") } else { None }) {
        let _ = bubble_win.hide();
    }

    let _ = app.emit("messenger-auth-status", serde_json::json!({
        "accountIndex": idx,
        "connected": false
    }));

    Ok(())
}

/// Command tải lại trang Messenger cho tài khoản chỉ định
#[tauri::command]
pub fn refresh_messenger(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let label = get_messenger_window_label(idx);
    println!("[BE - Messenger] Tải lại phiên Messenger {:02}", idx);
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

    if let Some(chat_window) = win {
        let _ = chat_window.eval("window.location.reload();");
    }
    Ok(())
}

/// Command đóng và hủy hẳn cửa sổ webview của tài khoản bị xóa, xóa sạch cache/storage
#[tauri::command]
pub fn close_messenger_window(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let label = get_messenger_window_label(idx);
    println!("[BE - Storage] Đóng cửa sổ và dọn dẹp profile cho Messenger {:02}", idx);
    state.clear_account_auth(idx);

    // 1. Huỷ cửa sổ chat webview
    let win = app.get_webview_window(&label)
        .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

    if let Some(chat_window) = win {
        let _ = chat_window.hide();
        let _ = chat_window.destroy();
    }

    if idx == 1 {
        if let Some(old) = app.get_webview_window("chat-messenger") {
            let _ = old.hide();
            let _ = old.destroy();
        }
    }

    // 2. Ẩn cửa sổ bubble tương ứng (giữ nguyên handle để có thể show lại tức thì khi cần)
    let bubble_label = get_bubble_window_label(idx);
    if let Some(bw) = app.get_webview_window(&bubble_label) {
        let _ = bw.hide();
    }
    if idx == 1 {
        if let Some(bw) = app.get_webview_window("bubble") {
            let _ = bw.hide();
        }
    }

    // 3. Xoá triệt để thư mục profile/storage trên ổ cứng (User Data Directory)
    // Chạy ngắt tiến trình msedgewebview2 phụ thuộc trước để nhả file lock
    let target_needle = format!("messenger_{:02}", idx);
    let app_handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));

        #[cfg(target_os = "windows")]
        {
            let kill_cmd = format!(
                "Get-CimInstance Win32_Process | Where-Object {{ $_.Name -like '*msedgewebview2*' -and $_.CommandLine -like '*{}*' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}",
                target_needle
            );
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &kill_cmd])
                .output();
        }

        std::thread::sleep(std::time::Duration::from_millis(150));

        if let Ok(mut profile_dir) = app_handle.path().app_data_dir() {
            profile_dir.push("profiles");
            profile_dir.push(&target_needle);
            if profile_dir.exists() {
                for _ in 0..5 {
                    if std::fs::remove_dir_all(&profile_dir).is_ok() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    });

    Ok(())
}

/// Command xóa toàn bộ bộ nhớ đệm, cookies và storage của một tài khoản cụ thể trên ổ cứng
#[tauri::command]
pub fn clear_account_storage(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    close_messenger_window(app, state, account_index)
}

/// Command kiểm tra trạng thái đăng nhập thực tế của tài khoản Messenger
#[tauri::command]
pub fn check_messenger_auth(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<serde_json::Value, String> {
    let idx = account_index.unwrap_or(1);
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

    let win = app.get_webview_window(&label)
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
            let _ = ensure_bubble_positioned_and_visible(&app, idx);
            let _ = app.emit("chat-visibility-changed", false);
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

/// Command nhận báo cáo trạng thái xác thực từ webview và đồng bộ sang UI
#[tauri::command]
pub fn report_auth_status(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    connected: bool,
    name: Option<String>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
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
        let _ = ensure_bubble_positioned_and_visible(&app, idx);
        let _ = app.emit("chat-visibility-changed", false);
    }

    let _ = app.emit("messenger-auth-status", serde_json::json!({
        "accountIndex": idx,
        "connected": connected,
        "name": name
    }));
    Ok(())
}
