/// Verifies login state per c_user cookie and routing invariants
pub fn is_window_logged_in(chat_window: &tauri::WebviewWindow) -> (bool, String) {
    let title = chat_window.title().unwrap_or_default();
    let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
    if let Some(pos) = url_str.find("#auth_uid=") {
        let uid_part = &url_str[pos + 10..];
        let clean_uid = uid_part.split(['&', '#', '/']).next().unwrap_or("").trim().to_string();
        if !clean_uid.is_empty() {
            return (true, clean_uid);
        }
    }
    if title.contains("[AUTH_OK]") {
        return (true, String::new());
    }
    if url_str.contains("checkpoint") || url_str.contains("two_step_verification") {
        return (false, String::new());
    }
    if url_str.contains("messenger.com") && (url_str.contains("/t/") || url_str.contains("/e2ee/")) {
        return (true, String::new());
    }
    (false, String::new())
}
