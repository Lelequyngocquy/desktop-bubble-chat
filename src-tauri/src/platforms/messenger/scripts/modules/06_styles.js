// ============================================================================
// MODULE 05: RESPONSIVE STYLING & COMPACT UI
// ============================================================================
// Nhiệm vụ:
// 1. Nhúng CSS tinh chỉnh giao diện Messenger phù hợp với kích thước cửa sổ popup (404x620).
// 2. Tự động ẩn panel thông tin bên phải khi chiều rộng màn hình hẹp (< 500px).

function injectMessengerStyles() {
  const styleId = "desktop-bubble-chat-style";
  if (!document.getElementById(styleId)) {
    const style = document.createElement("style");
    style.id = styleId;
    style.textContent = `
      /* Ẩn panel thông tin bên phải khi chiều rộng hẹp */
      @media (max-width: 500px) {
        div[role="complementary"] {
          display: none !important;
        }
      }
      /* Tối ưu hiệu năng render danh sách tin nhắn */
      div[role="grid"] div[role="row"], div[role="log"] > div {
        content-visibility: auto;
        contain-intrinsic-size: 0 44px;
      }
      /* Tạm dừng hiệu ứng CSS khi cửa sổ ẩn/unfocused */
      body.app-window-unfocused * {
        animation-play-state: paused !important;
      }
    `;
    if (document.head) {
      document.head.appendChild(style);
      logDebug("STYLE-INJECT", "✓ Đã tiêm CSS tối ưu giao diện compact vào document.head");
    } else {
      document.addEventListener("DOMContentLoaded", () => {
        if (document.head) {
          document.head.appendChild(style);
          logDebug("STYLE-INJECT", "✓ Đã tiêm CSS tối ưu giao diện compact khi DOMContentLoaded");
        }
      });
    }
  }
}
