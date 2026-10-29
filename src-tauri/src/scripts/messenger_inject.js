// Preload script inject vào Messenger Web (420x620)
(function () {
  // 1. Tự động trích xuất số tin nhắn chưa đọc từ document.title
  let lastBadgeCount = -1;

  function reportBadge() {
    try {
      const title = document.title || "";
      const match = title.match(/\((\d+)\+?\)/);
      const count = match ? parseInt(match[1], 10) : 0;

      if (count !== lastBadgeCount) {
        lastBadgeCount = count;
        if (window.__TAURI__ && window.__TAURI__.core) {
          window.__TAURI__.core.invoke("update_unread_badge", { count: count });
        }
      }
    } catch (e) {
      // Ignored
    }
  }

  // Lắng nghe thẻ <title> thay đổi nội dung
  const titleObserver = new MutationObserver(reportBadge);

  function initObserver() {
    const titleEl = document.querySelector("title");
    if (titleEl) {
      titleObserver.observe(titleEl, { childList: true, characterData: true, subtree: true });
    } else {
      setTimeout(initObserver, 500);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initObserver);
  } else {
    initObserver();
  }

  // Dự phòng định kỳ 2.5s kiểm tra 1 lần
  setInterval(reportBadge, 2500);

  // 2. Inject CSS thu gọn Messenger vừa vặn khung popup 420x620
  const style = document.createElement("style");
  style.id = "desktop-bubble-chat-style";
  style.textContent = `
    /* Ẩn panel thông tin bên phải khi chiều rộng hẹp */
    @media (max-width: 500px) {
      div[role="complementary"] {
        display: none !important;
      }
    }
  `;
  if (document.head) {
    document.head.appendChild(style);
  } else {
    document.addEventListener("DOMContentLoaded", () => {
      document.head && document.head.appendChild(style);
    });
  }
})();
