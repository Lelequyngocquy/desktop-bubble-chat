# HƯỚNG DẪN THỰC THI TỪNG BƯỚC (STEP-BY-STEP BUILD GUIDE)
> **Dự án:** DesktopBubbleChat (Tauri v2 + React + TypeScript + Windows 11)  
> **Nguyên tắc:** Mỗi bước là một module độc lập, hoàn thành bước nào kiểm thử ngay bước đó (`verify immediately`), không làm gộp tránh lỗi chồng chéo.

---

## 📋 BẢNG THEO DÕI TIẾN ĐỘ TỔNG THỂ

- [ ] **Giai đoạn 0:** Khởi tạo nền tảng & Cấu trúc dự án
- [ ] **Giai đoạn 1:** Cửa sổ Bong bóng nổi (UI, Kéo thả & Snap mép)
- [ ] **Giai đoạn 2:** Khay hệ thống (System Tray) & Vòng đời ứng dụng
- [ ] **Giai đoạn 3:** Cửa sổ Messenger & Cơ chế Đóng/Mở Thông minh
- [ ] **Giai đoạn 4:** Huy hiệu thông báo (Unread Badge) & Tinh chỉnh CSS
- [ ] **Giai đoạn 5:** Tối ưu hóa & Đóng gói cài đặt (.exe/.msi)

---

## 🛠️ CHI TIẾT TỪNG BƯỚC THỰC HIỆN

---

### GIAI ĐOẠN 0: KHỞI TẠO NỀN TẢNG & MÔI TRƯỜNG

#### Bước 0.1: Khởi tạo dự án Tauri v2 với template React + TypeScript
- **Mục tiêu:** Tạo khung dự án Tauri v2 kết hợp React + Vite + TypeScript hoàn chỉnh.
- **Tệp liên quan:** Toàn bộ thư mục gốc, `src/`, `src-tauri/`.
- **Nhiệm vụ:**
  1. Chạy lệnh tạo project hoặc tích hợp Tauri v2 vào thư mục hiện tại.
  2. Cấu hình biến môi trường Cargo/Rust trên máy.
  3. Kiểm tra file `package.json` và `src-tauri/Cargo.toml`.
- **Cách test ngay:**
  ```powershell
  $env:PATH += ";$env:USERPROFILE\.cargo\bin"
  npm run tauri dev
  ```
- **Tiêu chuẩn hoàn thành (DoD):** Cửa sổ mặc định của Tauri xuất hiện trên màn hình mà không báo lỗi build.

---

#### Bước 0.2: Cài đặt thư viện phụ trợ & Tích hợp Design Tokens
- **Mục tiêu:** Cài Tailwind CSS, Lucide Icons, Zustand và đưa `design_constant.txt` vào code TypeScript.
- **Tệp liên quan:**
  - `package.json`
  - `tailwind.config.js`, `postcss.config.js`, `src/index.css`
  - `src/constants/design.ts` (chuyển đổi từ `design_constant.txt`)
- **Nhiệm vụ:**
  1. Cài đặt các gói: `npm i lucide-react zustand` và devDependencies cho `tailwindcss postcss autoprefixer`.
  2. Chuyển đổi `design_constant.txt` thành `src/constants/design.ts` với đầy đủ định kiểu `as const`.
  3. Khởi tạo CSS reset và nạp font Inter nếu cần.
- **Cách test ngay:**
  - Import thử một biến màu từ `design.ts` vào `App.tsx` (ví dụ `Color.primary.main`) và dùng một class Tailwind để render nút bấm.
  - Chạy `npm run tauri dev` xem giao diện đổi màu theo token không.
- **Tiêu chuẩn hoàn thành (DoD):** Tailwind biên dịch được style và import hằng số không có lỗi TypeScript.

---

### GIAI ĐOẠN 1: BONG BÓNG NỔI (BUBBLE WINDOW)

#### Bước 1.1: Cấu hình cửa sổ Bong bóng trong suốt & Always On Top
- **Mục tiêu:** Biến cửa sổ chính thành khung tròn mini $64 \times 64$, trong suốt hoàn toàn, luôn nổi trên màn hình.
- **Tệp liên quan:**
  - `src-tauri/tauri.conf.json`
  - `src/index.css`
- **Nhiệm vụ:**
  1. Cấu hình cửa sổ label `"bubble"` trong `tauri.conf.json`:
     - `width: 64`, `height: 64`, `resizable: false`
     - `decorations: false`, `transparent: true`, `alwaysOnTop: true`, `skipTaskbar: true`.
  2. Thiết lập nền `body`, `html`, `#root` trong `src/index.css` trong suốt hoàn toàn (`background: transparent`).
- **Cách test ngay:**
  - Chạy `npm run tauri dev`.
  - Quan sát desktop: Phải thấy một khung nhỏ nổi trên các ứng dụng khác, không có viền tiêu đề Windows, nền trong suốt.
- **Tiêu chuẩn hoàn thành (DoD):** Cửa sổ $64 \times 64$ nổi trên mọi cửa sổ khác, không che desktop bằng viền vuông trắng.

---

#### Bước 1.2: Thiết kế UI Bong bóng & Huy hiệu (Bubble Component)
- **Mục tiêu:** Render icon Messenger bo tròn mềm mại, đổ bóng nhẹ và khung hiển thị số thông báo đỏ theo `design_constant.ts`.
- **Tệp liên quan:**
  - `src/components/Bubble.tsx`
  - `src/assets/messenger.svg`
  - `src/App.tsx`
- **Nhiệm vụ:**
  1. Thêm asset SVG logo Messenger vào thư mục `src/assets/`.
  2. Dựng component `Bubble`:
     - Nền tròn kích thước $64 \times 64$, viền mỏng (`border: 1px solid rgba(...)`), bóng đổ `drop-shadow`.
     - Huy hiệu đỏ tròn ở góc trên bên phải (chấm đỏ hoặc số badge).
     - Hiệu ứng hover & active mượt mà.
- **Cách test ngay:**
  - Mở app, nhìn trực quan bong bóng: Có viền tròn, icon rõ nét, badge hiển thị thử số "3".
- **Tiêu chuẩn hoàn thành (DoD):** Visual khớp thiết kế, không bị răng cưa hay vỡ khung viền.

---

#### Bước 1.3: Kéo thả bong bóng tự do (Window Dragging)
- **Mục tiêu:** Cho phép người dùng nhấn giữ chuột và kéo bong bóng di chuyển khắp mọi ngóc ngách màn hình.
- **Tệp liên quan:**
  - `src/hooks/useWindowDrag.ts`
  - `src/components/Bubble.tsx`
- **Nhiệm vụ:**
  1. Xử lý sự kiện kéo thả chuột (`mousedown`, `mousemove`, `mouseup`) hoặc gọi API native `appWindow.startDragging()`.
  2. Đảm bảo khi kéo không bị mất con trỏ chuột (sử dụng pointer capture hoặc native window dragging).
- **Cách test ngay:**
  - Dùng chuột nhấn vào bong bóng và kéo vòng quanh 4 góc màn hình.
  - Thử kéo nhanh xem bong bóng có bám theo chuột mượt mà không.
- **Tiêu chuẩn hoàn thành (DoD):** Bong bóng di chuyển trơn tru khắp màn hình khi kéo và dừng lại chính xác tại điểm thả.

---

#### Bước 1.4: Tự động hít vào mép màn hình (Snap to Edge)
- **Mục tiêu:** Khi thả chuột, bong bóng tự động trượt êm vào mép trái hoặc mép phải gần nhất của màn hình.
- **Tệp liên quan:**
  - `src/hooks/useSnapEdge.ts` (hoặc tính toán trong Rust / React kết hợp Tauri Monitor API)
- **Nhiệm vụ:**
  1. Lấy thông tin chiều rộng/cao của màn hình làm việc (Work Area Monitor bounds).
  2. Khi người dùng nhả chuột (`mouseup`):
     - Đo toạ độ $X$: Nếu vị trí gần mép trái hơn $\to$ snap về $X = 12px$. Nếu gần mép phải hơn $\to$ snap về $X = \text{Screen Width} - 76px$.
     - Giữ nguyên toạ độ $Y$ (nhưng giới hạn không để lọt ra ngoài mép trên/dưới Taskbar).
  3. Thêm animation trượt mượt mà (lerp/transition).
- **Cách test ngay:**
  - Kéo bong bóng ra giữa màn hình rồi thả tay $\to$ xem bong bóng có tự trượt vào mép trái/phải không.
  - Thử kéo lên sát mép trên hoặc mép dưới để kiểm tra việc không bị chìm mất khỏi màn hình.
- **Tiêu chuẩn hoàn thành (DoD):** Bong bóng luôn tự nép sát vào một bên mép màn hình một cách tự nhiên.

---

#### Bước 1.5: Thao tác kéo xuống mép đáy để ẩn (Dismiss to Tray)
- **Mục tiêu:** Kéo bong bóng xuống sát mép dưới màn hình thì bong bóng tự ẩn đi.
- **Tệp liên quan:**
  - `src/hooks/useWindowDrag.ts`
  - `src-tauri/src/lib.rs` (command ẩn cửa sổ)
- **Nhiệm vụ:**
  1. Kiểm tra toạ độ khi kéo: Nếu $Y > \text{Screen Height} - 80px$ lúc thả chuột.
  2. Kích hoạt hiệu ứng mờ dần và gọi Rust command để ẩn cửa sổ bong bóng (`window.hide()`).
- **Cách test ngay:**
  - Kéo bong bóng quẹt xuống sát mép đáy màn hình và thả tay $\to$ bong bóng biến mất.
- **Tiêu chuẩn hoàn thành (DoD):** Thả bong bóng ở đáy sẽ ẩn cửa sổ mà không gây treo app.

---

### GIAI ĐOẠN 2: KHAY HỆ THỐNG (SYSTEM TRAY) & VÒNG ĐỜI

#### Bước 2.1: Tạo System Tray Icon & Menu điều khiển
- **Mục tiêu:** Giúp người dùng gọi lại bong bóng sau khi đã ẩn, hoặc thoát ứng dụng hoàn toàn.
- **Tệp liên quan:**
  - `src-tauri/src/tray.rs` (hoặc cấu hình trong `lib.rs`)
  - `src-tauri/icons/`
- **Nhiệm vụ:**
  1. Khởi tạo Tray Icon bằng API Tauri v2 (`tauri::tray::TrayIconBuilder`).
  2. Tạo Menu:
     - "Hiện bong bóng" (Show Bubble)
     - "Tải lại trang" (Reload)
     - Dấu phân cách
     - "Thoát ứng dụng" (Exit)
  3. Bắt sự kiện click đúp vào Tray icon: Tự động hiện lại bong bóng và đặt lại vị trí an toàn.
- **Cách test ngay:**
  - Kéo bong bóng xuống đáy để ẩn.
  - Tìm icon app ở khay hệ thống (góc dưới bên phải Taskbar Windows).
  - Chuột phải chọn "Hiện bong bóng" $\to$ Bong bóng phải nổi lại ngay trên màn hình.
  - Chọn "Thoát" $\to$ Ứng dụng đóng sạch sẽ, tiến trình tắt hoàn toàn.
- **Tiêu chuẩn hoàn thành (DoD):** Tray hoạt động ổn định, gọi lại app bình thường.

---

### GIAI ĐOẠN 3: CỬA SỔ MESSENGER & CƠ CHẾ ĐÓNG/MỞ THÔNG MINH

#### Bước 3.1: Khởi tạo cửa sổ Messenger với WebView2 cố định Session
- **Mục tiêu:** Tạo cửa sổ thứ hai tải `https://www.messenger.com/` và lưu phiên đăng nhập vĩnh viễn.
- **Tệp liên quan:**
  - `src-tauri/src/windows/chat.rs`
  - `src-tauri/tauri.conf.json`
- **Nhiệm vụ:**
  1. Cấu hình hoặc tạo động WebviewWindow mang tên `"chat-messenger"`.
  2. Kích thước mặc định: $420 \times 620$, `decorations: false`, `skipTaskbar: true`, `alwaysOnTop: true`, `visible: false`.
  3. Cấu hình thư mục lưu profile dữ liệu WebView2 (UserDataFolder) cố định trong `%LOCALAPPDATA%`.
- **Cách test ngay:**
  - Tạm thời cho cửa sổ hiện lên (`visible: true`).
  - Đăng nhập tài khoản Messenger test.
  - Tắt hẳn app (Exit) rồi bật lại $\to$ Kiểm tra xem có giữ nguyên đăng nhập hay bị văng ra trang login.
- **Tiêu chuẩn hoàn thành (DoD):** Đăng nhập 1 lần, mở lại app vẫn giữ nguyên trạng thái đã đăng nhập.

---

#### Bước 3.2: Cơ chế Click Bong bóng $\leftrightarrow$ Bật/Tắt Khung Chat
- **Mục tiêu:** Nhấn vào bong bóng sẽ bật khung chat; nhấn lại bong bóng sẽ ẩn khung chat.
- **Tệp liên quan:**
  - `src/components/Bubble.tsx`
  - `src-tauri/src/commands.rs` (Command `toggle_chat_window`)
- **Nhiệm vụ:**
  1. Viết command Rust `toggle_chat(bubble_x, bubble_y)`:
     - Nếu `chat_window.is_visible() == true` $\to$ gọi `hide()`.
     - Nếu đang ẩn $\to$ tính vị trí và gọi `show()`, `set_focus()`.
  2. Phân biệt giữa hành động "Click" và "Kéo thả" trên bong bóng (nếu di chuyển chuột $> 5px$ thì không tính là Click).
- **Cách test ngay:**
  - Kéo di chuyển bong bóng: Khung chat không được vô tình nhảy ra.
  - Bấm nhẹ vào bong bóng: Khung chat hiện lên.
  - Bấm nhẹ lần nữa: Khung chat biến mất.
- **Tiêu chuẩn hoàn thành (DoD):** Nhận diện click nhạy bén, đóng/mở mượt mà.

---

#### Bước 3.3: Thuật toán Định vị Thông minh (Smart Positioning)
- **Mục tiêu:** Khung chat tự động bung về phía có khoảng trống, không bị tràn viền màn hình.
- **Tệp liên quan:**
  - `src-tauri/src/windows/chat.rs`
- **Nhiệm vụ:**
  1. Lấy toạ độ $(X_b, Y_b)$ của bong bóng và kích thước màn hình làm việc:
     - **Nếu bong bóng ở mép phải:** Mở khung chat về bên trái bong bóng ($X_{chat} = X_b - 420 - 10$).
     - **Nếu bong bóng ở mép trái:** Mở khung chat về bên phải bong bóng ($X_{chat} = X_b + 64 + 10$).
     - Căn chỉnh $Y_{chat}$ sao cho khung chat nằm ngang tầm bong bóng, nhưng nếu sát mép dưới màn hình thì đẩy lên trên để không tràn xuống thanh Taskbar.
  2. Áp dụng toạ độ qua `chat_window.set_position()`.
- **Cách test ngay:**
  - Kéo bong bóng qua mép trái $\to$ Bấm mở: Khung chat bung sang phải.
  - Kéo bong bóng qua mép phải $\to$ Bấm mở: Khung chat bung sang trái.
  - Kéo bong bóng xuống góc dưới cùng $\to$ Bấm mở: Toàn bộ khung chat vẫn nằm trọn trong màn hình.
- **Tiêu chuẩn hoàn thành (DoD):** Khung chat không bao giờ bị cắt cụt hay tràn ra ngoài màn hình.

---

#### Bước 3.4: Tự động ẩn khi nhấp chuột ra ngoài (Auto-hide on Blur)
- **Mục tiêu:** Khi khung chat đang mở, người dùng bấm vào ứng dụng khác hoặc desktop thì khung chat tự ẩn.
- **Tệp liên quan:**
  - `src-tauri/src/lib.rs` (Window event handler)
- **Nhiệm vụ:**
  1. Lắng nghe sự kiện `WindowEvent::Focused(false)` của cửa sổ `chat-messenger`.
  2. Khi mất focus: Gọi `chat_window.hide()`.
  3. Lưu ý: Tránh xung đột khi người dùng click vào chính bong bóng để đóng (dùng debounce hoặc cờ trạng thái).
- **Cách test ngay:**
  - Mở khung chat lên.
  - Click chuột vào một vùng trống trên màn hình desktop hoặc cửa sổ trình duyệt khác.
  - Khung chat phải lập tức tự ẩn đi.
- **Tiêu chuẩn hoàn thành (DoD):** Tự ẩn êm, không gây nháy cửa sổ.

---

### GIAI ĐOẠN 4: HUY HIỆU THÔNG BÁO & TINH CHỈNH GIAO DIỆN

#### Bước 4.1: Bắt số tin nhắn chưa đọc từ tiêu đề trang (Title Observer)
- **Mục tiêu:** Trích xuất số unread từ tiêu đề trang Messenger (ví dụ: `(2) Messenger`).
- **Tệp liên quan:**
  - `src-tauri/src/scripts/preload.js` hoặc theo dõi Title trực tiếp từ Rust
- **Nhiệm vụ:**
  1. Theo dõi sự thay đổi của tiêu đề cửa sổ Messenger.
  2. Trích xuất số bằng Regex: `/\((\d+)\)/`.
  3. Gửi số unread về Rust core (hoặc Rust đọc title định kỳ/bằng Win32 event hook).
- **Cách test ngay:**
  - Dùng một tài khoản khác gửi tin nhắn đến tài khoản đang mở.
  - Quan sát log xem có nhận đúng số lượng tin nhắn chưa đọc (`unread = 1`, `unread = 2`) không.
- **Tiêu chuẩn hoàn thành (DoD):** Khi có tin nhắn đến, backend nhận được con số chính xác.

---

#### Bước 4.2: Đồng bộ số thông báo lên Bong bóng (Sync Badge to Bubble)
- **Mục tiêu:** Hiển thị chấm đỏ hoặc con số lên bong bóng nổi ngoài Desktop.
- **Tệp liên quan:**
  - `src-tauri/src/commands.rs` (hoặc `app_handle.emit_to("bubble", ...)`)
  - `src/components/Bubble.tsx`
  - `src/stores/useChatStore.ts`
- **Nhiệm vụ:**
  1. Rust phát event `badge-updated` sang cửa sổ `bubble`.
  2. Component `Bubble.tsx` lắng nghe qua `@tauri-apps/api/event`:
     - Nếu `count == 0`: Ẩn badge.
     - Nếu `count > 0` và `<= 99`: Hiển thị số cụ thể.
     - Nếu `count > 99`: Hiển thị `99+`.
- **Cách test ngay:**
  - Gửi tin nhắn thử hoặc gọi lệnh test event từ Rust.
  - Quan sát bong bóng: Badge đỏ nhảy số lập tức. Đọc tin nhắn xong badge tự biến mất.
- **Tiêu chuẩn hoàn thành (DoD):** Số badge hiển thị chuẩn xác thời gian thực.

---

#### Bước 4.3: Inject CSS thu gọn & Tối ưu hoá giao diện Messenger
- **Mục tiêu:** Làm Messenger hiển thị vừa vặn trong khung $420 \times 620$ mà không bị tràn hay lộ thanh cuộn thừa.
- **Tệp liên quan:**
  - `src-tauri/src/scripts/custom_messenger.css`
  - `src-tauri/src/windows/chat.rs`
- **Nhiệm vụ:**
  1. Viết CSS thu gọn thanh điều hướng bên trái (Icon bar), ẩn panel thông tin bên phải.
  2. Tự động co giãn danh sách đoạn chat và ô soạn thảo tin nhắn.
  3. Nạp đoạn CSS này vào Webview qua `initialization_script` hoặc inject khi trang tải xong.
- **Cách test ngay:**
  - Mở khung chat lên, giao diện Messenger vừa khít khung cửa sổ $420 \times 620$, nhìn như một ứng dụng native mini.
- **Tiêu chuẩn hoàn thành (DoD):** Bố cục không bị vỡ, trải nghiệm đọc/gửi tin nhắn thuận tiện.

---

### GIAI ĐOẠN 5: TỐI ƯU HOÁ & ĐÓNG GÓI SẢN PHẨM

#### Bước 5.1: Quản lý mức tiêu thụ tài nguyên & Tiết kiệm RAM
- **Mục tiêu:** Giảm thiểu xung đột tài nguyên khi khung chat đang ở trạng thái ẩn.
- **Tệp liên quan:**
  - `src-tauri/src/windows/chat.rs`
- **Nhiệm vụ:**
  1. Khi cửa sổ chat bị ẩn (`hide()`): Không tắt tiến trình để giữ nguyên phiên, nhưng cấu hình cờ tiết kiệm tài nguyên của WebView2 nếu cần.
  2. Kiểm tra bộ nhớ tổng thể của toàn bộ app.
- **Cách test ngay:**
  - Bật Task Manager, theo dõi mức RAM của `DesktopBubbleChat.exe` khi ẩn và khi hiện.
- **Tiêu chuẩn hoàn thành (DoD):** App chạy nền mượt mà, không bị memory leak hay giật lag máy.

#### Bước 5.2: Khởi động cùng Windows (Autostart Plugin)
- **Mục tiêu:** Cung cấp tuỳ chọn tự khởi động cùng máy tính khi bật máy.
- **Tệp liên quan:**
  - `src-tauri/Cargo.toml` (`tauri-plugin-autostart`)
  - `src-tauri/src/lib.rs`
- **Nhiệm vụ:**
  1. Thêm plugin `tauri-plugin-autostart`.
  2. Bổ sung mục chọn "Khởi động cùng Windows" vào Tray Menu.
- **Cách test ngay:**
  - Bật tuỳ chọn trong Tray menu $\to$ Mở Task Manager $\to$ tab Startup Apps xem đã xuất hiện app chưa.
- **Tiêu chuẩn hoàn thành (DoD):** Tắt bật tính năng startup thành công.

#### Bước 5.3: Đóng gói bản cài đặt độc lập (.exe / .msi)
- **Mục tiêu:** Tạo bộ cài đặt hoàn chỉnh để có thể chạy trên bất kỳ máy Windows 11 nào.
- **Tệp liên quan:**
  - `src-tauri/tauri.conf.json` (phần `bundle`)
- **Nhiệm vụ:**
  1. Cấu hình icon ứng dụng đầy đủ các kích thước.
  2. Chạy lệnh đóng gói:
     ```powershell
     $env:PATH += ";$env:USERPROFILE\.cargo\bin"
     npm run tauri build
     ```
- **Cách test ngay:**
  - Vào `src-tauri/target/release/bundle/nsis/` hoặc `msi/`, chạy file cài đặt trên máy.
  - Thử gỡ cài đặt và cài lại kiểm tra tính toàn vẹn.
- **Tiêu chuẩn hoàn thành (DoD):** File cài đặt chạy trơn tru, biểu tượng app hiển thị chuẩn, không phụ thuộc vào môi trường dev.

---

## 📌 QUY TẮC PHÁT TRIỂN & GIẢI QUYẾT LỖI
1. **Luôn kiểm tra PATH Cargo:** Trước khi chạy lệnh build, đảm bảo đường dẫn `$env:USERPROFILE\.cargo\bin` có trong PATH.
2. **Không sửa nhiều thứ cùng lúc:** Chỉ chuyển sang bước tiếp theo khi bước hiện tại đã pass `DoD`.
3. **Giữ cấu hình tối giản:** Tránh cài quá nhiều plugin thừa; chỉ dùng đúng các crate cần thiết cho tính năng MVP.
