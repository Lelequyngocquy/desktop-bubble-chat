// Preload script inject vào Messenger Web (420x620)
(function () {
  // 1. Giữ mọi chuyển hướng (popup, OAuth, Captcha, "Continue as...") bên trong cùng cửa sổ
  try {
    window.open = function (url) {
      if (url && typeof url === "string") {
        window.location.href = url;
      }
      return window;
    };
  } catch (e) {}

  // 2. Tự động kiểm tra trạng thái xác thực và gắn thẻ vào document.title
  const AUTH_TAG = " [AUTH_OK]";

  function checkAndTagAuthStatus() {
    try {
      const url = window.location.href || "";
      const pathname = window.location.pathname || "";
      const hostname = window.location.hostname || "";

      // Kiểm tra xem có đang ở trang login / checkpoint / xác minh không
      const hasPasswordInput = !!document.querySelector('input[type="password"]');
      const hasLoginButton = !!(
        document.querySelector('button[name="login"]') ||
        document.querySelector('#loginbutton') ||
        document.querySelector('[data-testid="royal_login_button"]')
      );
      const isLoginUrl =
        pathname.includes("/login") ||
        url.includes("/checkpoint") ||
        url.includes("facebook.com/login");

      const isLoginPage = hasPasswordInput || hasLoginButton || isLoginUrl;

      // Kiểm tra xem đã xuất hiện giao diện chat của Messenger chưa
      const hasChatElements = !!(
        document.querySelector('div[role="navigation"]') ||
        document.querySelector('div[role="main"]') ||
        document.querySelector('div[aria-label*="Đoạn chat"]') ||
        document.querySelector('div[aria-label*="Chats"]') ||
        document.querySelector('a[href*="/t/"]') ||
        document.querySelector('div[data-pagelet="LeftRail"]')
      );

      const isInboxPath = pathname.startsWith("/t/") || pathname.startsWith("/e2ee/");

      const isLoggedIn =
        hostname.includes("messenger.com") &&
        !isLoginPage &&
        (hasChatElements || isInboxPath || (!hasPasswordInput && !hasLoginButton && !isLoginUrl));

      let rawTitle = document.title || "Messenger";

      if (isLoggedIn) {
        if (!rawTitle.includes(AUTH_TAG)) {
          document.title = rawTitle + AUTH_TAG;
        }
      } else {
        if (rawTitle.includes(AUTH_TAG)) {
          document.title = rawTitle.replace(AUTH_TAG, "");
        }
      }
    } catch (e) {
      // Ignored
    }
  }

  // 3. Tự động trích xuất số tin nhắn chưa đọc từ document.title
  let lastBadgeCount = -1;

  function reportBadge() {
    try {
      checkAndTagAuthStatus();

      const rawTitle = document.title || "";
      const cleanTitle = rawTitle.replace(AUTH_TAG, "");
      const match = cleanTitle.match(/\((\d+)\+?\)/);
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
  const titleObserver = new MutationObserver(() => {
    reportBadge();
  });

  function initObserver() {
    const titleEl = document.querySelector("title");
    if (titleEl) {
      titleObserver.observe(titleEl, { childList: true, characterData: true, subtree: true });
    } else {
      setTimeout(initObserver, 500);
    }
    reportBadge();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initObserver);
  } else {
    initObserver();
  }

  // Kiểm tra định kỳ mỗi 1.2 giây
  setInterval(reportBadge, 1200);

  // 4. Inject CSS thu gọn Messenger vừa vặn khung popup 420x620
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
    `;
    if (document.head) {
      document.head.appendChild(style);
    } else {
      document.addEventListener("DOMContentLoaded", () => {
        document.head && document.head.appendChild(style);
      });
    }
  }
})();
