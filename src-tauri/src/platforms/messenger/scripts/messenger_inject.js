// Preload script inject vào Messenger Web (420x620)
(function () {
  const hostname = window.location.hostname || "";
  const pathname = window.location.pathname || "";
  const url = window.location.href || "";

  // 1. NẾU ĐANG Ở TRANG FACEBOOK HOẶC META (XÁC THỰC 2FA / CAPTCHA / CHECKPOINT / OAUTH):
  // TUYỆT ĐỐI KHÔNG CAN THIỆP ĐIỀU HƯỚNG, KHÔNG CHẶN CLICK, KHÔNG CHẶN WINDOW.OPEN, KHÔNG INJECT CSS!
  // Để nguyên bản 100% trang web chính thức của Meta xử lý luồng bảo mật.
  if (!hostname.includes("messenger.com")) {
    return;
  }

  if (window.__DESKTOP_BUBBLE_INJECTED__) return;
  window.__DESKTOP_BUBBLE_INJECTED__ = true;

  const TARGET_MESSENGER_URL = "https://www.messenger.com/";

  // 2. BẢO VỆ ĐIỀU HƯỚNG: Chỉ giữ trong messenger.com khi đang dùng tính năng chat
  function enforceMessengerUrl() {
    try {
      const curHost = window.location.hostname || "";
      const curPath = window.location.pathname || "";
      const curUrl = window.location.href || "";

      if (!curHost || curUrl === "about:blank") return false;

      // Cho phép các trang đăng nhập, 2FA, checkpoint của Facebook / Meta
      const isAuthFlow =
        curHost.includes("messenger.com") ||
        curHost.includes("facebook.com") ||
        curHost.includes("meta.com") ||
        curPath.includes("/login") ||
        curPath.includes("/checkpoint") ||
        curPath.includes("/two_step_verification") ||
        curUrl.includes("checkpoint") ||
        curUrl.includes("/login");

      if (!isAuthFlow) {
        window.location.replace(TARGET_MESSENGER_URL);
        return true;
      }
    } catch (e) {}
    return false;
  }

  // 3. Chặn window.open và popup ngoài khi đang chat Messenger thông thường
  try {
    window.open = function (openUrl) {
      if (!openUrl) return null;
      const urlStr = String(openUrl);
      try {
        const parsed = new URL(urlStr, window.location.origin);
        if (
          parsed.hostname.includes("messenger.com") ||
          parsed.hostname.includes("facebook.com") ||
          parsed.hostname.includes("meta.com")
        ) {
          window.location.href = urlStr;
          return window;
        }
      } catch (e) {}

      // Mở link ngoài bằng trình duyệt mặc định của hệ thống
      if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke("open_external_url", { url: urlStr }).catch(() => {});
      }
      return null;
    };
  } catch (e) {}

  // 4. Chặn click thẻ <a> chuyển hướng ra ngoài messenger.com
  document.addEventListener(
    "click",
    function (e) {
      const anchor = e.target && e.target.closest ? e.target.closest("a") : null;
      if (!anchor || !anchor.href) return;

      const href = anchor.href;
      if (href.startsWith("javascript:") || href.startsWith("#")) return;

      try {
        const parsed = new URL(href, window.location.origin);
        if (
          parsed.hostname.includes("messenger.com") ||
          parsed.hostname.includes("facebook.com") ||
          parsed.hostname.includes("meta.com")
        ) {
          return;
        }

        // Chặn không cho webview chuyển trang tới domain xa lạ
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

  // 5. Tự động kiểm tra trạng thái xác thực và gắn thẻ vào document.title
  const AUTH_OK_TAG = " [AUTH_OK]";
  const AUTH_LOGIN_TAG = " [AUTH_LOGIN]";

  function checkAndTagAuthStatus() {
    try {
      enforceMessengerUrl();

      const url = window.location.href || "";
      const pathname = window.location.pathname || "";
      const hostname = window.location.hostname || "";

      // Kiểm tra xem có đang ở trang login / checkpoint rõ ràng không
      const isLoginUrl =
        pathname.includes("/login") ||
        url.includes("/checkpoint") ||
        url.includes("facebook.com/login");

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

      const titleLower = (document.title || "").toLowerCase();
      const isLoginTitle =
        titleLower.includes("log in to facebook") ||
        titleLower.includes("log into facebook") ||
        titleLower.includes("đăng nhập vào facebook");

      // Đã đăng nhập: Trên messenger.com, không ở URL login/checkpoint, và có giao diện chat hoặc ở trang chủ Messenger
      const isLoggedIn =
        hostname.includes("messenger.com") &&
        !isLoginUrl &&
        !isLoginTitle &&
        (hasChatElements || isInboxPath || pathname === "/" || pathname === "");

      let rawTitle = document.title || "Messenger";
      rawTitle = rawTitle.replace(AUTH_OK_TAG, "").replace(AUTH_LOGIN_TAG, "").trim();

      if (isLoggedIn) {
        if (!document.title.includes(AUTH_OK_TAG)) {
          document.title = rawTitle + AUTH_OK_TAG;
        }
        if (window.__TAURI__ && window.__TAURI__.core) {
          window.__TAURI__.core.invoke("report_auth_status", { connected: true }).catch(() => {});
        }
      } else if (isLoginUrl || isLoginTitle) {
        if (!document.title.includes(AUTH_LOGIN_TAG)) {
          document.title = rawTitle + AUTH_LOGIN_TAG;
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
      const cleanTitle = rawTitle.replace(AUTH_OK_TAG, "").replace(AUTH_LOGIN_TAG, "").trim();
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
