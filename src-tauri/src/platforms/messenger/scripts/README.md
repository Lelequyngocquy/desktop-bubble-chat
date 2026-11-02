# Messenger Web Injected Scripts & Architecture

Thư mục này chứa toàn bộ các script can thiệp vào tiến trình Webview2 của Messenger Web (`https://www.messenger.com/`).

---

## 1. Cấu Trúc Module (`modules/`)

Để đảm bảo tính độc lập trách nhiệm (Single Responsibility Principle - SRP), dễ dàng bảo trì và tối ưu hóa hiệu năng, mã nguồn inject chỉ tập trung vào các trách nhiệm cốt lõi: Xác thực danh tính người dùng (Login Auth), Bảo vệ điều hướng (Navigation Guard), Tối ưu bố cục giao diện (Responsive Styles), và Xử lý mạng (Offline Resilience):

| Module | File | Trách nhiệm chính |
| :--- | :--- | :--- |
| **01** | [`01_ipc_bridge.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/01_ipc_bridge.js) | Cầu nối `invokeTauri` (Tauri v2 Core & `__TAURI_INTERNALS__`) và hàm `logDebug` chuẩn hóa đẩy log về Rust Backend. |
| **02** | [`02_identity_auth.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/02_identity_auth.js) | Quét cookie `c_user` và Client Runtime để lấy Facebook UID; giám sát luồng xác thực / 2FA / Checkpoint; đánh giá trạng thái login (`checkAndReportAuth`), đóng dấu `[AUTH_OK]` và hash `#auth_uid=`. |
| **03** | [`03_navigation_guard.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/03_navigation_guard.js) | Bảo vệ điều hướng: ép URL ở `messenger.com` khi đã login; chặn `window.open` và click thẻ `<a>` ngoài domain để mở bằng trình duyệt hệ thống (`open_external_url`). |
| **04** | [`04_offline_handler.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/04_offline_handler.js) | Giám sát sự kiện `online` / `offline` của trình duyệt; hiển thị banner cảnh báo mất kết nối kèm nút thử lại; tự động reload khi phục hồi mạng. |
| **05** | [`05_styles.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/05_styles.js) | Nhúng CSS responsive thu gọn layout Messenger vừa vặn khung cửa sổ chat popup (404x620), tự động ẩn panel thông tin bên phải (`div[role="complementary"]`). |
| **06** | [`06_lifecycle.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/06_lifecycle.js) | Bộ điều phối vòng đời: cơ chế `runAdaptiveInterval` tự động giãn cách chu kỳ khi cửa sổ ẩn (`document.hidden`) để tối ưu hóa CPU và RAM; khởi tạo toàn bộ module khi DOM sẵn sàng. |

---

## 2. Cơ Chế Tích Hợp Vào Rust Backend (Compile-time Concat)

Trong [`src-tauri/src/platforms/messenger/constants.rs`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/constants.rs), Rust sử dụng macro `concat!(...)` kết hợp `include_str!(...)` để nhúng trực tiếp toàn bộ 6 module vào hằng số `MESSENGER_INJECT_SCRIPT` tại thời điểm biên dịch:

```rust
pub const MESSENGER_INJECT_SCRIPT: &str = concat!(
    "(function () {\n",
    "  const TARGET_MESSENGER_URL = \"https://www.messenger.com/\";\n\n",
    include_str!("scripts/modules/01_ipc_bridge.js"),
    "\n\n",
    include_str!("scripts/modules/02_identity_auth.js"),
    "\n\n",
    include_str!("scripts/modules/03_navigation_guard.js"),
    "\n\n",
    include_str!("scripts/modules/04_offline_handler.js"),
    "\n\n",
    include_str!("scripts/modules/05_styles.js"),
    "\n\n",
    include_str!("scripts/modules/06_lifecycle.js"),
    "\n})();\n"
);
```

### Lợi ích kỹ thuật:
1. **Tinh gọn & Bảo mật cao**: Không còn can thiệp bắt trộm tin nhắn hay cào DOM cây tin nhắn.
2. **Single Source of Truth (SSOT)**: Khi sửa đổi bất kỳ module nào trong `modules/`, Rust biên dịch lại lập tức phản ánh thay đổi mà không cần chạy bất kỳ lệnh bundle JavaScript thủ công nào.
3. **Zero Runtime Overhead**: Việc ghép chuỗi diễn ra 100% ở compile time trong Rust binary.
