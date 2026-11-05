# DesktopBubbleChat

[![Platform](<https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(x64)-0078D6?style=flat-square&logo=windows>)](https://www.microsoft.com/windows)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![React 19](https://img.shields.io/badge/React-19.1-61DAFB?style=flat-square&logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-DEA584?style=flat-square&logo=rust&logoColor=black)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)

> Ứng dụng bong bóng chat (Chat Heads) nổi mượt mà, tiện lợi trên Windows Desktop. Hỗ trợ đa nền tảng, đồ họa vector Win32 + Skia, hút mép vật lý thông minh và tương tác lướt chuột zero-click.

[English](README.md) | **Tiếng Việt**

---

**DesktopBubbleChat** mang trải nghiệm bong bóng chat (Chat Heads) mượt mà, tiện lợi từ thiết bị di động lên hệ điều hành Windows 10 và 11. Được xây dựng trên nền tảng **Tauri v2**, **React 19**, **TypeScript** và lõi native **Rust (Win32 + Skia)**, ứng dụng tách biệt hoàn toàn giữa engine vật lý hiển thị và adapter của từng nền tảng nhắn tin. Dự án cho phép chạy đồng thời nhiều tài khoản độc lập trong các sandbox WebView2 riêng biệt, mang lại trải nghiệm tương tác trực quan và mượt mà trên desktop.

### Mục lục

- [Minh họa](#minh-họa)
- [Tính năng nổi bật](#tính-năng-nổi-bật)
- [Tổng quan kiến trúc](#tổng-quan-kiến-trúc)
- [Lộ trình phát triển](#lộ-trình-phát-triển)
- [Cài đặt & Khởi chạy](#cài-đặt--khởi-chạy)
- [Phím tắt & Khay hệ thống](#phím-tắt--khay-hệ-thống)
- [Mô hình bảo mật](#mô-hình-bảo-mật)
- [Xử lý sự cố thường gặp](#xử-lý-sự-cố-thường-gặp)
- [Đóng góp & Giấy phép](#đóng-góp--giấy-phép)

---

## Minh họa

![Minh họa](assets/gif/preview.gif)

### Tính năng nổi bật

- **Kiến trúc Adapter cắm rút (Pluggable)**: Lõi engine vật lý và render giao diện tách biệt hoàn toàn với logic của từng mạng xã hội, dễ dàng mở rộng sang các nền tảng mới.
- **Hỗ trợ đa tài khoản độc lập**: Đăng nhập và duy trì tối đa 5 tài khoản cùng lúc trên mỗi nền tảng mà không lo xung đột cookie hay session.
- **Lõi đồ họa Vector Rust Win32 + Skia**: Luồng native riêng biệt vẽ icon và huy hiệu thông báo chưa đọc với chất lượng đồ họa khử răng cưa mượt mà.
- **Hút mép màn hình & Thu gọn thông minh**: Tự động hít vào mép màn hình gần nhất khi thả chuột; tự ẩn 75% diện tích bong bóng vào cạnh màn hình khi không tương tác để giải phóng không gian làm việc.
- **Bộ giải quyết xung đột vị trí (Collision Resolver)**: Tự động sắp xếp khoảng cách theo chiều dọc, tránh việc các bong bóng đè lên nhau ở cùng một mép màn hình.
- **Tương tác lướt chuột không cần click (Zero-Click)**:
  - **Rê chuột 100ms** vào bong bóng để trượt ra và tự động mở cửa sổ chat.
  - **Rời chuột 1000ms** để tự động đóng cửa sổ và đưa bong bóng về chế độ ẩn nép.
- **Âm thanh và Thông báo Native**: Phát âm thanh hệ thống Win32 khi có tin nhắn và hiển thị toast notification từ Windows Action Center.
- **Phím tắt toàn cục & Khay hệ thống (System Tray)**: Ẩn/hiện nhanh toàn bộ bong bóng với phím tắt `Ctrl + Shift + M`.

---

### Tổng quan kiến trúc

Ứng dụng kết hợp giữa frontend dashboard hiện đại và engine native hiệu năng cao:

- **Giao diện Dashboard (React 19 + Tailwind)**: Quản lý kết nối tài khoản, cấu hình vật lý bong bóng, theo dõi chỉ số.
- **Chat Head Windows (Rust + Skia + Win32)**: Các cửa sổ nổi không viền, trong suốt, vẽ vector trực tiếp qua API Win32 Layered Window.
- **Chat Window (Microsoft Edge WebView2)**: Mỗi tài khoản mở trong một sandbox dữ liệu riêng biệt (`profiles/<platform>_<index>`).

---

### Lộ trình phát triển

| Nền tảng               | Hình thức tích hợp                | Đa tài khoản                       | Trạng thái   |
| :--------------------- | :-------------------------------- | :--------------------------------- | :----------- |
| **Facebook Messenger** | WebView2 độc lập + Rust Daemon    | Tối đa 5 tài khoản (Profile 01–05) | **Khả dụng** |
| **Telegram**           | Web / TDesktop Adapter            | Hỗ trợ nhiều tài khoản (Dự kiến)   | **Kế hoạch** |
| **Zalo**               | Web Wrapper + Trình theo dõi Auth | Hỗ trợ nhiều tài khoản (Dự kiến)   | **Kế hoạch** |
| **Discord**            | Web Client / Gateway Adapter      | Hỗ trợ nhiều tài khoản (Dự kiến)   | **Kế hoạch** |

---

### Cài đặt & Khởi chạy

#### Yêu cầu môi trường

- **Hệ điều hành**: Windows 10 (1903+) hoặc Windows 11 (64-bit)
- **Node.js**: `>= 18.0.0`
- **Rust & Cargo**: Bản Stable (1.80+ với target `x86_64-pc-windows-msvc`)
- **Visual Studio Build Tools 2022** (Gói C++ workload để lấy `link.exe`)
- **Microsoft Edge WebView2 Runtime** (Có sẵn trên Win 10/11)

#### Các bước cài đặt

1. **Clone mã nguồn**:

   ```bash
   git clone https://github.com/Lelequyngocquy/desktop-bubble-chat.git
   cd desktop-bubble-chat
   ```

2. **Cài đặt dependencies**:

   ```bash
   npm install
   ```

3. **Kiểm tra tương thích môi trường**:

   ```powershell
   npm run app:check
   ```

4. **Chạy ở chế độ Development**:

   ```powershell
   npm run app:start
   ```

5. **Đóng gói bản cài đặt Production**:
   ```powershell
   npm run tauri build
   ```

---

### Phím tắt & Khay hệ thống

- **`Ctrl + Shift + M`**: Phím tắt bật/tắt hiển thị toàn bộ bong bóng chat trên màn hình.
- **Menu chuột phải System Tray**:
  - Mở / Đóng Trang chủ (Main Dashboard)
  - Bật / Ẩn tất cả bong bóng
  - Bật / Tắt hiển thị từng tài khoản riêng biệt
  - Thoát ứng dụng an toàn
- **Giám sát tài nguyên**:
  Kiểm tra mức chiếm dụng RAM/CPU của các profile WebView2:
  ```powershell
  powershell -ExecutionPolicy Bypass -File scripts/status.ps1 -Watch
  ```

---

### Mô hình bảo mật

1. **Không sử dụng máy chủ trung gian**: Dữ liệu tin nhắn truyền trực tiếp giữa máy cục bộ của bạn và máy chủ chính thức của nền tảng (ví dụ: `messenger.com`). Không thu thập telemetry hay chuyển tiếp cookie ra bên ngoài.
2. **Cách ly Profile tuyệt đối**: Dữ liệu mỗi tài khoản được lưu trong thư mục riêng (`profiles/<platform>_<index>`). Cookie, token đăng nhập và cache hoàn toàn tách biệt.
3. **Cơ chế phân quyền Tauri v2 chặt chẽ**: Quy định chi tiết trong file `src-tauri/capabilities/default.json`, ngăn chặn webview thực thi các lệnh shell hoặc truy cập tệp tin trái phép.

---

### Xử lý sự cố thường gặp

- **Lỗi cổng 1420 đang bị chiếm dụng (`Port 1420 is already in use`)**: Chạy lệnh `Stop-Process -Name "DesktopBubbleChat" -Force -ErrorAction SilentlyContinue` hoặc khởi động bằng `npm run app:start` (script sẽ tự động giải phóng port).
- **Lỗi khoá file `msedgewebview2.exe` khi build**: Do tiến trình WebView2 cũ chưa tắt hẳn. Nhấp chuột phải vào khay hệ thống chọn **Thoát ứng dụng**, hoặc gõ lệnh PowerShell:
  ```powershell
  Get-Process msedgewebview2 -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like "*com.desktopbubblechat.app*" } | Stop-Process -Force
  ```
- **Lỗi thiếu linker Rust (`link.exe not found`)**: Cần cài đặt **Visual Studio Build Tools 2022** và tích chọn mục **Desktop development with C++**.

---

## Maintainers

- **Le Ngoc Quy** ([@Lelequyngocquy](https://github.com/Lelequyngocquy))

---

## Đóng góp & Phát triển (Contributing)

Mọi đóng góp, báo lỗi (issue) và yêu cầu tính năng mới đều được hoan nghênh:

1. Fork dự án
2. Tạo nhánh tính năng (`git checkout -b feat/tinh-nang-moi`)
3. Commit thay đổi tuân theo Conventional Commits (`git commit -m "feat: mo ta tinh nang"`)
4. Push lên nhánh của bạn (`git push origin feat/tinh-nang-moi`)
5. Tạo Pull Request

---

## Giấy phép (License)

Dự án được phân phối dưới giấy phép **MIT License** - xem chi tiết tại file [LICENSE](LICENSE).
