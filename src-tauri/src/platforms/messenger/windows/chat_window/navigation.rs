use tauri::Url;

/// Decision returned by native WebView navigation policy for Messenger windows
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationDecision {
    Allow,
    OpenExternal(String),
    Block,
}

/// Checks if host matches exact domain or subdomain
fn host_matches(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{}", domain))
}

/// Checks if URL is a Meta outbound Link Shim (/l.php or /flx/warn)
pub fn is_meta_link_shim(url: &Url) -> bool {
    let Some(host) = url.host_str().map(|h| h.to_ascii_lowercase()) else {
        return false;
    };
    let path = url.path().to_ascii_lowercase();
    if host == "l.messenger.com" || host == "lm.messenger.com" || host == "l.facebook.com" || host == "lm.facebook.com" {
        return true;
    }
    (host_matches(&host, "messenger.com") || host_matches(&host, "facebook.com")) && (path.starts_with("/l.php") || path.starts_with("/flx/warn"))
}

/// Extracts clean destination URL from up to 2 nested Meta Link Shim layers
pub fn extract_destination_url(raw_url: &str) -> String {
    let mut current = raw_url.trim().to_string();
    for _ in 0..2 {
        let Ok(parsed) = Url::parse(&current) else {
            break;
        };
        if !is_meta_link_shim(&parsed) {
            break;
        }
        let target_opt = parsed.query_pairs().find(|(k, _)| k == "u").map(|(_, v)| v.into_owned());
        let Some(target) = target_opt else {
            break;
        };
        let trimmed = target.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            current = trimmed.to_string();
            continue;
        }
        break;
    }
    current
}

/// Checks if URL points to a genuine Meta attachment/media download CDN
pub fn is_download_url(url: &Url) -> bool {
    let scheme = url.scheme();
    if scheme == "blob" || scheme == "data" {
        return true;
    }
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let Some(host) = url.host_str().map(|h| h.to_ascii_lowercase()) else {
        return false;
    };
    let path = url.path().to_ascii_lowercase();
    if host_matches(&host, "fbsbx.com") || host_matches(&host, "fbcdn.net") {
        return true;
    }
    if path.contains("file_download.php") || path.contains("attachment_preview") {
        return true;
    }
    let has_dl_param = url.query_pairs().any(|(k, v)| k == "dl" && v == "1");
    has_dl_param && host_matches(&host, "messenger.com")
}

/// Checks if URL is a Meta internal utility/SSO/auth path safe for in-webview loading
pub fn is_meta_auth_or_sso_url(url: &Url) -> bool {
    let Some(host) = url.host_str().map(|h| h.to_ascii_lowercase()) else {
        return false;
    };
    let path = url.path().to_ascii_lowercase();
    if host == "challenges.cloudflare.com" || host == "staticxx.facebook.com" || host.starts_with("accountscenter.") || host_matches(&host, "facebook.net") {
        return true;
    }
    if host_matches(&host, "meta.com") {
        return path.starts_with("/login") || path.starts_with("/auth") || path.starts_with("/checkpoint") || path.starts_with("/oidc") || path.contains("two_step_verification");
    }
    if host_matches(&host, "facebook.com") {
        let auth_prefixes = [
            "/login",
            "/checkpoint",
            "/two_step_verification",
            "/security/2fac",
            "/recover",
            "/device-based",
            "/save-device",
            "/oidc",
            "/dialog/oauth",
            "/x/oauth",
            "/messenger_sync",
            "/cookie/",
            "/common/referer_frame.php",
            "/platform/xd_arbiter",
            "/plugins/",
            "/connect/",
            "/tr/",
        ];
        if auth_prefixes.iter().any(|p| path.starts_with(p)) {
            return true;
        }
        if path.contains("/dialog/oauth") || path.contains("two_step_verification") {
            return true;
        }
    }
    false
}

/// Checks if URL belongs to Meta auth ecosystem during explicit Auth Mode
fn is_allowed_in_auth_mode(url: &Url) -> bool {
    if is_meta_link_shim(url) {
        return false;
    }
    let Some(host) = url.host_str().map(|h| h.to_ascii_lowercase()) else {
        return false;
    };
    host_matches(&host, "messenger.com")
        || host_matches(&host, "facebook.com")
        || host_matches(&host, "meta.com")
        || host_matches(&host, "fbcdn.net")
        || host_matches(&host, "fbsbx.com")
        || host_matches(&host, "facebook.net")
        || host == "challenges.cloudflare.com"
}

/// Evaluates top-level or subframe WebView navigation policy for Messenger windows
pub fn decide_navigation(url: &Url, in_auth_mode: bool) -> NavigationDecision {
    let scheme = url.scheme();
    if matches!(scheme, "about" | "blob" | "data") {
        return NavigationDecision::Allow;
    }
    if scheme != "http" && scheme != "https" {
        return NavigationDecision::Block;
    }
    if is_meta_link_shim(url) {
        let dest = extract_destination_url(url.as_str());
        if let Ok(dest_url) = Url::parse(&dest) {
            if is_download_url(&dest_url) {
                return NavigationDecision::Allow;
            }
            if !is_meta_link_shim(&dest_url)
                && dest_url.host_str().is_some_and(|h| host_matches(&h.to_ascii_lowercase(), "messenger.com"))
            {
                return NavigationDecision::Allow;
            }
        }
        return NavigationDecision::OpenExternal(dest);
    }
    if is_download_url(url) {
        return NavigationDecision::Allow;
    }
    if in_auth_mode {
        if is_allowed_in_auth_mode(url) {
            return NavigationDecision::Allow;
        }
        return NavigationDecision::OpenExternal(url.to_string());
    }
    if let Some(host) = url.host_str().map(|h| h.to_ascii_lowercase())
        && host_matches(&host, "messenger.com")
    {
        return NavigationDecision::Allow;
    }
    if is_meta_auth_or_sso_url(url) {
        return NavigationDecision::Allow;
    }
    NavigationDecision::OpenExternal(url.to_string())
}

/// Evaluates new-window (target="_blank" / window.open) policy for Messenger windows
pub fn decide_new_window(url: &Url, in_auth_mode: bool) -> NavigationDecision {
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return NavigationDecision::Block;
    }
    let dest = extract_destination_url(url.as_str());
    let parsed_dest = Url::parse(&dest).unwrap_or_else(|_| url.clone());
    if is_download_url(&parsed_dest) {
        return NavigationDecision::OpenExternal(dest);
    }
    if in_auth_mode && is_allowed_in_auth_mode(&parsed_dest) {
        return NavigationDecision::Allow;
    }
    if !in_auth_mode && !is_meta_link_shim(url)
        && let Some(host) = parsed_dest.host_str().map(|h| h.to_ascii_lowercase())
        && host_matches(&host, "messenger.com")
    {
        let path = parsed_dest.path().to_ascii_lowercase();
        if path.starts_with("/groupcall") || path.starts_with("/call") || path.starts_with("/videocall") {
            return NavigationDecision::Allow;
        }
    }
    NavigationDecision::OpenExternal(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Url {
        Url::parse(s).expect("valid test url")
    }

    #[test]
    fn test_shared_facebook_posts_open_externally_in_chat_mode() {
        let cases = [
            "https://www.facebook.com/share/v/1AbCdEfGh/",
            "https://www.facebook.com/share/p/1XyZ999/",
            "https://www.facebook.com/share/r/1Reel999/",
            "https://www.facebook.com/reel/1234567890/",
            "https://www.facebook.com/watch/?v=999888",
            "https://www.facebook.com/photo/?fbid=12345&set=a.6789",
            "https://www.facebook.com/somepage/posts/pfbid02abc",
            "https://m.facebook.com/story.php?story_fbid=123&id=456",
            "https://www.facebook.com/groups/rustlang/permalink/12345/",
        ];
        for raw in cases {
            assert_eq!(decide_navigation(&parse(raw), false), NavigationDecision::OpenExternal(raw.to_string()), "failed for {}", raw);
            assert_eq!(decide_new_window(&parse(raw), false), NavigationDecision::OpenExternal(raw.to_string()), "new_window failed for {}", raw);
        }
    }

    #[test]
    fn test_link_shims_unwrap_and_open_externally() {
        let shim_v = "https://l.messenger.com/l.php?u=https%3A%2F%2Fwww.facebook.com%2Fshare%2Fv%2F1AbCdEfGh%2F&h=AT123";
        assert_eq!(decide_navigation(&parse(shim_v), false), NavigationDecision::OpenExternal("https://www.facebook.com/share/v/1AbCdEfGh/".to_string()));
        let www_shim = "https://www.messenger.com/l.php?u=https%3A%2F%2Fwww.facebook.com%2Fshare%2Fp%2F123%2F";
        assert_eq!(decide_navigation(&parse(www_shim), false), NavigationDecision::OpenExternal("https://www.facebook.com/share/p/123/".to_string()));
        let flx_warn = "https://www.facebook.com/flx/warn/?u=https%3A%2F%2Fexample.com%2Farticle";
        assert_eq!(decide_navigation(&parse(flx_warn), false), NavigationDecision::OpenExternal("https://example.com/article".to_string()));
        let fb_shim = "https://l.facebook.com/l.php?u=https%3A%2F%2Fgithub.com%2Ftauri-apps%2Ftauri";
        assert_eq!(decide_new_window(&parse(fb_shim), false), NavigationDecision::OpenExternal("https://github.com/tauri-apps/tauri".to_string()));
    }

    #[test]
    fn test_internal_messenger_and_downloads_allowed() {
        let allowed = [
            "https://www.messenger.com/",
            "https://www.messenger.com/t/1234567890",
            "https://www.messenger.com/e2ee/t/9876543210",
            "https://messenger.com/t/1234567890",
            "https://cdn.fbsbx.com/v/t59.2708-21/12345_file.pdf?_nc_cat=1&dl=1",
            "https://attachment.fbsbx.com/file_download.php?id=123",
            "https://scontent.fsgn5-1.fna.fbcdn.net/v/t1.15752-9/photo.jpg",
            "about:blank",
            "blob:https://www.messenger.com/550e8400-e29b-41d4-a716-446655440000",
        ];
        for raw in allowed {
            assert_eq!(decide_navigation(&parse(raw), false), NavigationDecision::Allow, "should allow {}", raw);
        }
    }

    #[test]
    fn test_meta_auth_sso_and_utility_iframes_allowed() {
        let allowed_auth_or_iframe = [
            "https://www.facebook.com/login/?next=https%3A%2F%2Fwww.messenger.com%2F",
            "https://www.facebook.com/checkpoint/?next=https%3A%2F%2Fwww.messenger.com%2F",
            "https://www.facebook.com/two_step_verification/authentication/",
            "https://www.facebook.com/oidc/?client_id=123",
            "https://www.facebook.com/common/referer_frame.php",
            "https://staticxx.facebook.com/x/connect/xd_arbiter/",
            "https://challenges.cloudflare.com/cdn-cgi/challenge-platform/",
        ];
        for raw in allowed_auth_or_iframe {
            assert_eq!(decide_navigation(&parse(raw), false), NavigationDecision::Allow, "should allow auth/iframe {}", raw);
        }
        assert_eq!(decide_navigation(&parse("https://www.facebook.com/anything"), true), NavigationDecision::Allow);
        assert_eq!(decide_navigation(&parse("https://example.com/terms"), true), NavigationDecision::OpenExternal("https://example.com/terms".to_string()));
    }

    #[test]
    fn test_new_window_in_chat_mode_opens_facebook_externally_even_if_login_path() {
        let fb_login = "https://www.facebook.com/login/";
        assert_eq!(decide_new_window(&parse(fb_login), false), NavigationDecision::OpenExternal(fb_login.to_string()));
        let call_url = "https://www.messenger.com/groupcall/LINK:abc123/";
        assert_eq!(decide_new_window(&parse(call_url), false), NavigationDecision::Allow);
    }
}
