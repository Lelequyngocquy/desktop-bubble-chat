# Messenger Web Injected Scripts & Architecture

Thư mục này chứa toàn bộ các script can thiệp vào tiến trình Webview2 của Messenger Web (`https://www.messenger.com/`).

---

## 1. Cấu Trúc Module (`modules/`)

Để đảm bảo tính độc lập trách nhiệm (Single Responsibility Principle - SRP), dễ dàng bảo trì, mở rộng và sửa lỗi mà không làm ảnh hưởng đến các thành phần khác, mã nguồn inject được chia tách thành 10 module độc lập:

| Module | File | Trách nhiệm chính |
| :--- | :--- | :--- |
| **01** | [`01_ipc_bridge.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/01_ipc_bridge.js) | Cầu nối `invokeTauri` (Tauri v2 Core & `__TAURI_INTERNALS__`), bộ đệm khử trùng lặp tin nhắn thời gian thực (sliding window 5000ms), hàm `reportMessageSafely`. |
| **02** | [`02_network_hooks.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/02_network_hooks.js) | Monkey-patch `window.fetch` và `XMLHttpRequest` để bóc tách gói tin GraphQL và MQTT-over-WebSocket; trích xuất `unread_count`, deltas tin nhắn và danh sách hội thoại. |
| **03** | [`03_notification_hooks.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/03_notification_hooks.js) | Giả lập W3C Notification API (`MockNotification`) và ServiceWorker để bắt tin nhắn mới không dựa vào DOM; đánh chặn âm thanh thông báo qua `HTMLMediaElement.prototype.play`. |
| **04** | [`04_identity_auth.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/04_identity_auth.js) | Quét cookie `c_user` và Client Runtime để lấy Facebook UID; xử lý luồng xác thực / 2FA / Checkpoint trên domain Meta; đánh giá trạng thái login (`checkAndReportAuth`), đóng dấu `[AUTH_OK]` và hash `#auth_uid=`. |
| **05** | [`05_navigation_guard.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/05_navigation_guard.js) | Bảo vệ điều hướng: ép URL ở `messenger.com` khi đã login; chặn `window.open` và click thẻ `<a>` ngoài domain để mở bằng trình duyệt hệ thống (`open_external_url`). |
| **06** | [`06_badge_observer.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/06_badge_observer.js) | Theo dõi thẻ `<title>` bằng `MutationObserver`, bóc tách số tin nhắn chưa đọc dạng `(N)` và tên người gửi nhấp nháy; gửi IPC `update_unread_badge`. |
| **07** | [`07_dom_fallback.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/07_dom_fallback.js) | Quét DOM trích xuất danh sách hội thoại gần nhất (threadId, sender, body, avatarUrl) làm kênh dự phòng khi network hook bị chặn; gửi `sync_recent_conversations`. |
| **08** | [`08_offline_handler.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/08_offline_handler.js) | Giám sát sự kiện `online` / `offline` của trình duyệt; hiển thị banner cảnh báo mất kết nối kèm nút thử lại; tự động reload khi phục hồi mạng. |
| **09** | [`09_styles.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/09_styles.js) | Nhúng CSS responsive thu gọn layout Messenger vừa vặn khung cửa sổ chat popup (404x620), tự động ẩn panel thông tin bên phải (`div[role="complementary"]`). |
| **10** | [`10_lifecycle.js`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/scripts/modules/10_lifecycle.js) | Bộ điều phối vòng đời: cơ chế `runAdaptiveInterval` tự động giãn cách chu kỳ khi cửa sổ ẩn (`document.hidden`) để tối ưu hóa CPU và RAM; khởi tạo toàn bộ module khi DOM sẵn sàng. |

---

## 2. Cơ Chế Tích Hợp Vào Rust Backend (Compile-time Concat)

Trong [`src-tauri/src/platforms/messenger/constants.rs`](file:///D:/DesktopBubbleChat/src-tauri/src/platforms/messenger/constants.rs), Rust sử dụng macro `concat!(...)` kết hợp `include_str!(...)` để nhúng trực tiếp toàn bộ 10 module vào hằng số `MESSENGER_INJECT_SCRIPT` tại thời điểm biên dịch:

```rust
pub const MESSENGER_INJECT_SCRIPT: &str = concat!(
    "(function () {\n",
    "  const TARGET_MESSENGER_URL = \"https://www.messenger.com/\";\n\n",
    include_str!("scripts/modules/01_ipc_bridge.js"),
    "\n\n",
    include_str!("scripts/modules/02_network_hooks.js"),
    "\n\n",
    // ...
    "\n})();\n"
);
```

### Lợi ích kỹ thuật:
1. **Single Source of Truth (SSOT)**: Khi sửa đổi bất kỳ module nào trong `modules/`, Rust biên dịch lại lập tức phản ánh thay đổi mà không cần chạy bất kỳ lệnh bundle JavaScript thủ công nào.
2. **Zero Runtime Overhead**: Việc ghép chuỗi diễn ra 100% ở compile time trong Rust binary.
3. **Dễ kiểm thử và bảo trì**: Mỗi module có phạm vi rõ ràng, tách biệt hoàn toàn giữa Network, Auth, DOM, Notification và IPC.
