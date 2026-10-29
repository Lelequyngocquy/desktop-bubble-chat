/// Trích xuất số unread từ tiêu đề cửa sổ Messenger (ví dụ: "(3) Messenger", "(99+) Messenger")
pub fn parse_unread_from_title(title: &str) -> u32 {
    let clean_title = title.replace("[AUTH_OK]", "");
    if let (Some(start), Some(end)) = (clean_title.find('('), clean_title.find(')')) {
        if start < end {
            let inner = &clean_title[start + 1..end];
            let digits: String = inner.chars().filter(|c| c.is_ascii_digit()).collect();
            return digits.parse::<u32>().unwrap_or(0);
        }
    }
    0
}
