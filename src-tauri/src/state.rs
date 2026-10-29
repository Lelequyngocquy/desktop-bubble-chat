use std::sync::atomic::{AtomicBool, AtomicI64};

/// Trạng thái theo dõi thời điểm ẩn cửa sổ và chế độ xác thực
pub struct ChatState {
    pub last_blur_time: AtomicI64,
    pub is_auth_mode: AtomicBool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            last_blur_time: AtomicI64::new(0),
            is_auth_mode: AtomicBool::new(false),
        }
    }
}
