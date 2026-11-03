/// Trích xuất số unread từ tiêu đề cửa sổ Messenger (ví dụ: "(3) Messenger", "(99+) Messenger")
pub fn parse_unread_from_title(title: &str) -> u32 {
    let clean_title = title.replace("[AUTH_OK]", "").replace("[AUTH_LOGIN]", "");
    if let (Some(start), Some(end)) = (clean_title.find('('), clean_title.find(')')) {
        if start < end {
            let inner = &clean_title[start + 1..end];
            let digits: String = inner.chars().filter(|c| c.is_ascii_digit()).collect();
            return digits.parse::<u32>().unwrap_or(0);
        }
    }
    0
}

/// Trích xuất tên người gửi từ tiêu đề cửa sổ Messenger nếu có (ví dụ: "(1) Nguyễn Văn A", "(1) Nguyễn Văn A | Messenger")
pub fn parse_sender_from_title(title: &str) -> Option<String> {
    let clean_title = title.replace("[AUTH_OK]", "").replace("[AUTH_LOGIN]", "");
    if let (Some(start), Some(end)) = (clean_title.find('('), clean_title.find(')')) {
        if start < end {
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
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sender_from_title() {
        assert_eq!(
            parse_sender_from_title("(1) Nguyễn Văn A | Messenger"),
            Some("Nguyễn Văn A".to_string())
        );
        assert_eq!(
            parse_sender_from_title("(2) John Doe - Messenger"),
            Some("John Doe".to_string())
        );
        assert_eq!(
            parse_sender_from_title("(1) Bạn B đã gửi một tin nhắn."),
            Some("Bạn B".to_string())
        );
        assert_eq!(parse_sender_from_title("(3) Messenger"), None);
        assert_eq!(parse_sender_from_title("Messenger"), None);
    }
}
