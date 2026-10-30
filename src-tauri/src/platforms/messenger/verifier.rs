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
