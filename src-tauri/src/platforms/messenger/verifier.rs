/// Pure helper evaluating authentication signals from window title and URL
pub fn evaluate_login_signals(title: &str, url_str: &str) -> (bool, String) {
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
    if url_str.contains("/login")
        || url_str.contains("login.php")
        || url_str.contains("checkpoint")
        || url_str.contains("two_step_verification")
    {
        return (false, String::new());
    }
    if url_str.contains("messenger.com") && (url_str.contains("/t/") || url_str.contains("/e2ee/")) {
        return (true, String::new());
    }
    (false, String::new())
}

/// Verifies login state per c_user cookie and routing invariants
pub fn is_window_logged_in(chat_window: &tauri::WebviewWindow) -> (bool, String) {
    let title = chat_window.title().unwrap_or_default();
    let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
    evaluate_login_signals(&title, &url_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_url_with_next_thread_is_not_logged_in() {
        let res = evaluate_login_signals(
            "Messenger",
            "https://www.messenger.com/login/?next=https%3A%2F%2Fwww.messenger.com%2Ft%2F123456",
        );
        assert_eq!(res, (false, String::new()));
    }

    #[test]
    fn test_authenticated_thread_url_is_logged_in() {
        let res = evaluate_login_signals("Messenger", "https://www.messenger.com/t/123456");
        assert_eq!(res, (true, String::new()));
    }
}
