/// Extracts unread count from Messenger window title prefix (e.g. "(3) Messenger", "(99+) Messenger")
pub fn parse_unread_from_title(title: &str) -> u32 {
    let clean_title = title.replace("[AUTH_OK]", "").replace("[AUTH_LOGIN]", "").replace("[PINNED]", "");
    let trimmed = clean_title.trim_start();
    if let Some(rest) = trimmed.strip_prefix('(')
        && let Some(end) = rest.find(')') {
            let inner = rest[..end].trim();
            let after = rest[end + 1..].trim();
            if inner.is_empty() || !inner.chars().all(|c| c.is_ascii_digit() || c == '+') {
                return 0;
            }
            let digits: String = inner.chars().filter(|c| c.is_ascii_digit()).collect();
            let is_direct_messenger = after.eq_ignore_ascii_case("Messenger");
            let is_valid_sender_badge = digits.len() <= 2
                && (after.contains("Messenger")
                    || after.contains("sent a message")
                    || after.contains("đã gửi"));
            if is_direct_messenger || is_valid_sender_badge {
                return digits.parse::<u32>().unwrap_or(0);
            }
        }
    0
}

/// Extracts sender name from window title if present (e.g. "(1) Alice | Messenger")
pub fn parse_sender_from_title(title: &str) -> Option<String> {
    let clean_title = title.replace("[AUTH_OK]", "").replace("[AUTH_LOGIN]", "").replace("[PINNED]", "");
    if let (Some(start), Some(end)) = (clean_title.find('('), clean_title.find(')'))
        && start < end {
            let after = clean_title[end + 1..].trim();
            if after.is_empty() || after.eq_ignore_ascii_case("Messenger") {
                return None;
            }
            let stripped = after
                .split('|')
                .next()
                .unwrap_or(after)
                .split('-')
                .next()
                .unwrap_or(after)
                .trim();
            let sender = stripped
                .replace("đã gửi một tin nhắn.", "")
                .replace("đã gửi một tin nhắn", "")
                .replace("sent a message.", "")
                .replace("sent a message", "")
                .trim()
                .to_string();
            if !sender.is_empty() && !sender.eq_ignore_ascii_case("Messenger") {
                return Some(sender);
            }
        }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sender_from_title() {
        assert_eq!(
            parse_sender_from_title("(1) Alice | Messenger"),
            Some("Alice".to_string())
        );
        assert_eq!(
            parse_sender_from_title("(2) John Doe - Messenger"),
            Some("John Doe".to_string())
        );
        assert_eq!(
            parse_sender_from_title("(1) Bob sent a message."),
            Some("Bob".to_string())
        );
        assert_eq!(parse_sender_from_title("(3) Messenger"), None);
        assert_eq!(parse_sender_from_title("Messenger"), None);
    }

    #[test]
    fn test_parse_unread_from_title_ignores_group_year_and_mid_parens() {
        assert_eq!(parse_unread_from_title("(3) Messenger"), 3);
        assert_eq!(parse_unread_from_title("(3) Messenger [PINNED]"), 3);
        assert_eq!(parse_unread_from_title("(99+) Messenger"), 99);
        assert_eq!(parse_unread_from_title("(12) Alice | Messenger"), 12);
        assert_eq!(parse_unread_from_title("(2026) Team"), 0);
        assert_eq!(parse_unread_from_title("(2026) Team | Messenger"), 0);
        assert_eq!(parse_unread_from_title("Team (2026) | Messenger"), 0);
    }
}
