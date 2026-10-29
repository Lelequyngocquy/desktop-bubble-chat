use std::time::{SystemTime, UNIX_EPOCH};

/// Lấy mốc thời gian hiện tại theo mili-giây (UNIX timestamp)
pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
