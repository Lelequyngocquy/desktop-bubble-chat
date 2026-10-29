# PLAN.md — DesktopBubbleChat (Tauri v2, Windows 11)

> **Mục tiêu:** Ứng dụng chat dạng bong bóng nổi ("chat heads") trên Windows 11. Phiên bản đầu tiên tập trung làm việc trực tiếp với **Messenger** thông qua Smart WebView Wrapper, chuẩn bị sẵn kiến trúc để mở rộng cho Telegram, Zalo và Discord sau này.

---

## 1. Mục tiêu sản phẩm

- [ ] Bong bóng tròn $64 \times 64$ luôn nổi trên mọi ứng dụng khác (`alwaysOnTop`), tự do kéo thả khắp màn hình.
- [ ] Bấm vào bong bóng → mở khung chat Messenger ngay cạnh; bấm ra ngoài hoặc bấm lại bong bóng → tự động thu về.
- [ ] Lưu phiên đăng nhập Messenger vĩnh viễn (không bắt đăng nhập lại khi tắt/mở app).
- [ ] Đọc số tin nhắn chưa đọc (unread badge) từ Messenger và hiển thị badge đỏ lên bong bóng nổi ngoài màn hình.
- [ ] Kéo bong bóng xuống đáy màn hình để đóng vào khay hệ thống (System Tray).
- [ ] Bộ nhớ nhẹ: WebView2 chạy nền mượt mà, RAM tối ưu bằng cách quản lý trạng thái ẩn/hiện hợp lý.
- [ ] Đóng gói thành file cài `.msi` / `.exe` chạy trơn tru trên Windows 11.

---

## 2. Phạm vi phiên bản (MVP: Messenger First)

### MVP (Phải hoàn thành)
- [ ] **Bong bóng (`bubble`):** Kích thước tròn, icon Messenger, badge số tin chưa đọc, kéo thả tự do, hút vào mép màn hình (snap to edge), vùng xóa ở đáy màn hình.
- [ ] **Khung chat (`chat-messenger`):** Tải trực tiếp `https://www.messenger.com/`, lưu cache/session profile WebView2.
- [ ] **Cầu nối dữ liệu (IPC bridge):**
  - Inject script bắt sự kiện đổi tiêu đề trang `document.title` để trích xuất số tin chưa đọc.
  - Bắn event về Rust, Rust cập nhật số badge lên cửa sổ bong bóng React.
- [ ] **Tối ưu hiển thị Pop-up:** Inject CSS tùy biến để Messenger hiển thị gọn ghẽ trên kích thước popup ($420 \times 620$).
- [ ] **System Tray:** Menu mở lại bong bóng, bật/tắt khởi động cùng Windows, thoát hoàn toàn.
- [ ] **Định vị thông minh:** Tự tính toán vị trí khung chat so với mép màn hình để không bị tràn ra ngoài display bounds.

### Mở rộng sau MVP (Giai đoạn Multi-Platform)
- [ ] Thêm các platform tiếp theo: Telegram (`web.telegram.org`), Zalo (`chat.zalo.me`), Discord (`discord.com/app`).
- [ ] Khung chuyển đổi nhanh (tab bar) giữa các nền tảng hoặc Master Hub gom nhiều bong bóng.
- [ ] Chế độ Quick Reply (trả lời nhanh qua input mini cạnh bong bóng).
- [ ] Phím tắt toàn cục (Global Shortcut) để ẩn/hiện bong bóng tức thì (ví dụ: `Ctrl + Shift + M`).

---

## 3. Kiến trúc kỹ thuật

### 3.1. Mô hình 2 cửa sổ (Multi-Window Architecture)

```
┌─────────────────────────────────────────────────────────────┐
│                      Rust Core (Tauri v2)                   │
│ - Quản trị vòng đời cửa sổ (Bubble & Chat)                  │
│ - IPC Bus trung chuyển dữ liệu Badge & Vị trí               │
│ - System Tray, Global Hotkeys, Autostart                    │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
       Tauri Events / Commands                 │ Webview Data Partition
               │                               │ (Persistence)
               ▼                               ▼
    ┌────────────────────┐          ┌──────────────────────┐
    │  Cửa sổ BUBBLE     │          │  Cửa sổ CHAT         │
    │  - React + Vite    │          │  - WebView2 Native   │
    │  - Kích thước 64x64│          │  - https://messenger │
    │  - Avatar + Badge  │          │  - Injected Script   │
    │  - Drag & Drop     │          │  - Responsive CSS    │
    └────────────────────┘          └──────────────────────┘
```

### 3.2. Cấu hình cửa sổ trong `tauri.conf.json`

**1. Cửa sổ `bubble` (Bong bóng nổi):**
```json
{
  "label": "bubble",
  "title": "DesktopBubbleChat - Bubble",
  "url": "index.html",
  "width": 64,
  "height": 64,
  "transparent": true,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "shadow": false
}
```

**2. Cửa sổ `chat-messenger` (Khung Messenger Web):**
```json
{
  "label": "chat-messenger",
  "title": "Messenger",
  "url": "https://www.messenger.com/",
  "width": 420,
  "height": 620,
  "transparent": false,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": true,
  "visible": false
}
```

### 3.3. Cơ chế giao tiếp & Injection Script

1. **Khởi tạo Webview Messenger:** Thiết lập WebView2 Data Directory cố định trong AppData (`%LOCALAPPDATA%/DesktopBubbleChat/messenger_profile`) để lưu trữ cookie phiên làm việc.
2. **Preload Script (`messenger_injected.js`):**
   - Lắng nghe thay đổi của tiêu đề tài liệu (`MutationObserver` trên `document.querySelector('title')`).
   - Messenger Web cập nhật title dạng `(3) Messenger` khi có 3 tin nhắn chưa đọc.
   - Regex parse số `count`:
     ```javascript
     const match = document.title.match(/\((\d+)\)/);
     const count = match ? parseInt(match[1], 10) : 0;
     window.__TAURI__.core.invoke('update_unread_badge', { count });
     ```
3. **Rust Dispatcher:**
   - Command `update_unread_badge` nhận số lượng `count` và phát event `emit_to("bubble", "badge-updated", count)`.
   - Frontend React trên cửa sổ `bubble` cập nhật số hiển thị trên chấm đỏ.
4. **Định vị & Focus:**
   - Khi bấm vào bong bóng: React gửi vị trí hiện tại $(x, y)$ của bong bóng về Rust.
   - Rust tính toán toạ độ cho khung chat (ưu tiên mở phía trên hoặc bên trái/phải tuỳ khoảng trống màn hình), sau đó gọi `window.set_position()`, `window.show()`, và `window.set_focus()`.
   - Lắng nghe sự kiện `focused(false)` trên cửa sổ chat: nếu mất focus (người dùng click ra chỗ khác) → tự động gọi `window.hide()`.

---

## 4. Công nghệ sử dụng

| Thành phần | Lựa chọn | Lý do |
|---|---|---|
| **Core Framework** | **Tauri v2** | Quản lý đa cửa sổ nhẹ, hỗ trợ trực tiếp WebView2 trên Windows 11 |
| **Bubble UI** | **React + Vite + Tailwind CSS** | Dựng animation bóng nổi, badge và hiệu ứng kéo thả mượt mà |
| **State (Bubble)** | **Zustand** | Quản lý trạng thái mở/đóng, toạ độ, số badge |
| **Icons & Style** | **Lucide Icons + Custom SVG** | Icon Messenger, nút thu nhỏ/đóng |
| **Target OS** | **Windows 11 (64-bit)** | Hệ điều hành tích hợp sẵn WebView2 Runtime |

---

## 5. Lộ trình triển khai (Chi tiết từng giai đoạn)

### Phase 1: Vỏ bong bóng & Khung điều khiển (2–3 ngày)
- [ ] Khởi tạo project Tauri v2 với template React + TypeScript.
- [ ] Cấu hình cửa sổ trong suốt `bubble` ($64 \times 64$, border-radius tròn, bóng đổ nhẹ).
- [ ] Làm chức năng kéo thả tự do bằng chuột (`onMouseDown`, `onMouseMove`, `onMouseUp`).
- [ ] Thêm tính năng **Snap to Edge**: Thả chuột gần viền màn hình thì bong bóng tự hút sát vào mép trái/phải.
- [ ] Vùng hủy (vùng tròn chữ X màu đỏ ở đáy màn hình xuất hiện khi kéo bong bóng). Thả vào đó sẽ ẩn app về System Tray.
- [ ] **Nghiệm thu Phase 1:** Bong bóng nổi di chuyển mượt mà trên các ứng dụng khác, thu về Tray chuẩn xác.

### Phase 2: Cửa sổ Messenger & Giữ phiên đăng nhập (2 ngày)
- [ ] Tạo cửa sổ thứ hai `chat-messenger` trỏ tới `https://www.messenger.com/`.
- [ ] Cấu hình data persistence cho WebView2 để giữ phiên đăng nhập tài khoản.
- [ ] Cài đặt logic tương tác:
  - Bấm bong bóng $\to$ mở Messenger.
  - Bấm lại bong bóng hoặc click ra ngoài cửa sổ chat $\to$ ẩn Messenger.
- [ ] Xử lý thuật toán vị trí mở:
  - Mặc định mở lệch sang trái hoặc phía trên bong bóng.
  - Nếu sát góc màn hình thì tự lật hướng để không bị che khuất.
- [ ] **Nghiệm thu Phase 2:** Đăng nhập Messenger một lần, đóng mở app vẫn giữ nguyên đăng nhập; bấm bong bóng bung khung chat chuẩn vị trí.

### Phase 3: Badge thông báo & Tinh chỉnh giao diện Messenger (2–3 ngày)
- [ ] Viết đoạn JavaScript inject vào `chat-messenger` để theo dõi `document.title`.
- [ ] Rust command nhận unread count và chuyển tiếp sang cửa sổ `bubble`.
- [ ] Hiển thị badge đỏ trên bong bóng (`99+` nếu số lượng quá lớn).
- [ ] Inject CSS để tối ưu giao diện:
  - Ẩn bớt thanh điều hướng thừa nếu kích thước cửa sổ quá hẹp.
  - Thêm một mini title-bar tự chế (nút ghim, nút thu nhỏ, nút tải lại trang).
- [ ] **Nghiệm thu Phase 3:** Khi có tin nhắn đến trên Messenger, bong bóng ngoài desktop lập tức nhảy số thông báo đỏ.

### Phase 4: Hoàn thiện & Đóng gói (1–2 ngày)
- [ ] Tích hợp tính năng chạy cùng Windows (Autostart plugin).
- [ ] Tối ưu RAM: thiết lập cơ chế giải phóng tài nguyên khi cửa sổ chat ở trạng thái ẩn trong thời gian dài.
- [ ] Build file cài đặt `.exe` / `.msi` bằng Tauri Bundler.
- [ ] Kiểm thử trên máy tính Windows 11 độc lập.

---

## 6. Xử lý rủi ro kỹ thuật

| Rủi ro | Giải pháp kỹ thuật |
|---|---|
| **Meta thay đổi cấu trúc web Messenger** | Không can thiệp sâu vào DOM nội dung tin nhắn; chỉ đọc unread count qua `document.title` (cực kỳ ổn định và hiếm khi đổi). |
| **Mất phiên đăng nhập sau khi restart** | Đặt đường dẫn WebView2 UserDataFolder cố định thông qua cấu hình môi trường Tauri hoặc Webview builder của Rust. |
| **Giao diện Messenger bị vỡ layout** | Đặt `min-width: 380px` cho cửa sổ chat và inject custom CSS để tự động ẩn sidebar bên phải (thông tin cuộc trò chuyện) khi chiều rộng cửa sổ $< 500px$. |
| **Cửa sổ chat không tự ẩn khi click ra ngoài** | Lắng nghe event `tauri::WindowEvent::Focused(false)` ở backend Rust và lập tức gọi `window.hide()`. |

---

## 7. Cấu trúc thư mục dự kiến

```
DesktopBubbleChat/
├── src/                          # Frontend React cho Bong bóng
│   ├── assets/
│   │   └── messenger.svg         # Icon Messenger
│   ├── components/
│   │   ├── Bubble.tsx            # Component bong bóng tròn & badge
│   │   └── DismissArea.tsx       # Vùng xóa ở đáy màn hình
│   ├── hooks/
│   │   └── useDragSnap.ts        # Hook xử lý kéo thả & snap mép
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Điểm khởi chạy app
│   │   ├── lib.rs                # Setup Tauri app & plugins
│   │   ├── windows/
│   │   │   ├── bubble.rs         # Logic quản lý cửa sổ bong bóng
│   │   │   └── chat.rs           # Logic tạo & định vị cửa sổ Messenger
│   │   └── scripts/
│   │       ├── preload.js        # Script inject bắt badge & title
│   │       └── style.css         # CSS inject tinh chỉnh layout Messenger
│   ├── capabilities/             # Phân quyền Tauri v2
│   ├── tauri.conf.json           # Cấu hình 2 cửa sổ & permissions
│   └── Cargo.toml
└── PLAN.md
```
