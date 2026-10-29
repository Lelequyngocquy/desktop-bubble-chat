// Preload script inject vào Messenger Web (420x620)
(function () {
  const TARGET_MESSENGER_URL = "https://www.messenger.com/";

  // 1. BẢO VỆ ĐIỀU HƯỚNG: Luôn giữ trong messenger.com, tự động chuyển về Messenger khi gặp URL lạ hoặc Facebook
  function enforceMessengerUrl() {
    try {
      const hostname = window.location.hostname || "";
      const pathname = window.location.pathname || "";
      const url = window.location.href || "";

      if (!hostname || url === "about:blank") return false;

      // Cho phép các trang checkpoint / 2FA hợp lệ nếu đang đăng nhập
      const is2FA = hostname.includes("facebook.com") && (
        pathname.includes("/checkpoint") ||
        url.includes("checkpoint") ||
        pathname.includes("/two_step_verification")
      );

      // Nếu không phải domain messenger.com và không phải trang 2FA -> Chuyển hướng ngay về Messenger
      if (!hostname.includes("messenger.com") && !is2FA) {
        window.location.replace(TARGET_MESSENGER_URL);
        return true;
      }
    } catch (e) {}
    return false;
  }

  // Thực thi kiểm tra ngay khi script nạp vào trang
  if (enforceMessengerUrl()) {
    return;
  }

  // 2. Chặn window.open và popup ngoài: Chỉ mở nội bộ nếu là messenger.com / 2FA
  try {
    window.open = function (url) {
      if (!url) return null;
      const urlStr = String(url);
      try {
        const parsed = new URL(urlStr, window.location.origin);
        if (parsed.hostname.includes("messenger.com") || parsed.pathname.includes("/checkpoint")) {
          window.location.href = urlStr;
          return window;
        }
      } catch (e) {}

      // Mở link ngoài / bài viết Facebook bằng trình duyệt mặc định của hệ thống
      if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke("open_external_url", { url: urlStr }).catch(() => {});
      }
      return null;
    };
  } catch (e) {}

  // 3. Chặn click thẻ <a> chuyển hướng ra ngoài messenger.com
  document.addEventListener(
    "click",
    function (e) {
      const anchor = e.target && e.target.closest ? e.target.closest("a") : null;
      if (!anchor || !anchor.href) return;

      const href = anchor.href;
      if (href.startsWith("javascript:") || href.startsWith("#")) return;

      try {
        const parsed = new URL(href, window.location.origin);
        if (parsed.hostname.includes("messenger.com") || parsed.pathname.includes("/checkpoint")) {
          return;
        }

        // Chặn không cho webview chuyển trang tới URL lạ hoặc Facebook
        e.preventDefault();
        e.stopPropagation();

        // Mở ra trình duyệt ngoài
        if (window.__TAURI__ && window.__TAURI__.core) {
          window.__TAURI__.core.invoke("open_external_url", { url: href }).catch(() => {});
        }
      } catch (err) {}
    },
    true
  );

  // 4. Tự động kiểm tra trạng thái xác thực và gắn thẻ vào document.title
  const AUTH_TAG = " [AUTH_OK]";

  function checkAndTagAuthStatus() {
    try {
      enforceMessengerUrl();

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

  // 5. Tự động trích xuất số tin nhắn chưa đọc từ document.title
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

  // Kiểm tra định kỳ mỗi 800ms
  setInterval(reportBadge, 800);
  setInterval(enforceMessengerUrl, 500);

  // 6. Inject CSS thu gọn Messenger vừa vặn khung popup 420x620
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
