// Preload script inject vào Messenger Web (420x620)
(function () {
  const TARGET_MESSENGER_URL = "https://www.messenger.com/";

  // Helper IPC an toàn: Hỗ trợ cả Global Tauri v2 và __TAURI_INTERNALS__
  function invokeTauri(cmd, args) {
    try {
      if (
        window.__TAURI__ &&
        window.__TAURI__.core &&
        typeof window.__TAURI__.core.invoke === "function"
      ) {
        return window.__TAURI__.core.invoke(cmd, args);
      }
      if (
        window.__TAURI_INTERNALS__ &&
        typeof window.__TAURI_INTERNALS__.invoke === "function"
      ) {
        return window.__TAURI_INTERNALS__.invoke(cmd, args);
      }
    } catch (e) {
      // IPC không khả dụng
    }
    return Promise.reject("Tauri IPC not available");
  }

  // --- TẦNG 1: NON-DOM INTERCEPTION VIA W3C NOTIFICATION API ---
  try {
    function MockNotification(title, options) {
      options = options || {};
      const accountIdx = window.__ACCOUNT_INDEX__ || 1;

      // Báo cáo tin nhắn mới trực tiếp về Tauri IPC <10ms
      invokeTauri("report_new_message", {
        senderName: title || "Tin nhắn mới",
        body: options.body || "",
        avatarUrl: options.icon || "",
        threadId: options.tag || "",
        accountIndex: accountIdx,
      }).catch(() => {});

      this.title = title;
      this.body = options.body || "";
      this.icon = options.icon || "";
      this.tag = options.tag || "";
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
    }

    MockNotification.permission = "granted";
    MockNotification.requestPermission = function (cb) {
      const res = Promise.resolve("granted");
      if (typeof cb === "function") cb("granted");
      return res;
    };
    MockNotification.maxActions = 2;

    MockNotification.prototype.close = function () {
      if (typeof this.onclose === "function") this.onclose();
    };
    MockNotification.prototype.addEventListener = function (type, listener) {
      this["on" + type] = listener;
    };
    MockNotification.prototype.removeEventListener = function (type) {
      this["on" + type] = null;
    };
    MockNotification.prototype.dispatchEvent = function () {
      return true;
    };

    Object.defineProperty(window, "Notification", {
      value: MockNotification,
      writable: true,
      configurable: true,
    });

    if (
      typeof ServiceWorkerRegistration !== "undefined" &&
      ServiceWorkerRegistration.prototype &&
      ServiceWorkerRegistration.prototype.showNotification
    ) {
      ServiceWorkerRegistration.prototype.showNotification = function (title, options) {
        new MockNotification(title, options);
        return Promise.resolve();
      };
    }
  } catch (e) {
    console.warn("[Messenger Inject] Notification interceptor error:", e);
  }

  // --- TẦNG 3: AUDIO CHIME HOOK (KÊNH BỔ TRỢ PHÁT HIỆN TIN NHẮN) ---
  try {
    const origAudioPlay = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      try {
        const isAudioTag = this instanceof HTMLAudioElement || this.tagName === "AUDIO";
        const hasSoundSrc = this.src && (this.src.includes(".mp3") || this.src.includes(".wav") || this.src.includes(".ogg"));
        if (isAudioTag || hasSoundSrc) {
          const accountIdx = window.__ACCOUNT_INDEX__ || 1;
          invokeTauri("log_frontend", {
            message: `[Audio Chime Intercept] Sound played on account ${accountIdx}`,
          }).catch(() => {});
        }
      } catch (err) {}
      return origAudioPlay.apply(this, arguments);
    };
  } catch (e) {}

  // 1. NHẬN DIỆN DANH TÍNH FACEBOOK/MESSENGER KHÔNG DỰA VÀO DOM (Industry Standard - c_user cookie):
  // Quét cookie 'c_user' (chứa UID số của Facebook, HttpOnly: false)
  // và Meta Client-side Runtime (CurrentUserInitialData / window.__user).
  function getFacebookUserId() {
    try {
      // a. Cookie c_user: Tiêu chuẩn vàng của hệ thống Meta Web
      const cookieMatch = document.cookie.match(/(?:^|;\s*)c_user=([0-9]+)/);
      if (cookieMatch && cookieMatch[1] && cookieMatch[1] !== "0") {
        return cookieMatch[1];
      }

      // b. Meta Client Module Runtime (require("CurrentUserInitialData").USER_ID)
      if (typeof window.require === "function") {
        try {
          const currentUser = window.require("CurrentUserInitialData");
          if (currentUser && currentUser.USER_ID && currentUser.USER_ID !== "0") {
            return String(currentUser.USER_ID);
          }
        } catch (e) {}
      }

      // c. Biến ngữ cảnh toàn cục window.__user hoặc Env.user_id
      if (window.__user && window.__user !== "0") {
        return String(window.__user);
      }
      if (window.Env && window.Env.user_id && window.Env.user_id !== "0") {
        return String(window.Env.user_id);
      }
    } catch (e) {}
    return null;
  }

  window.getFacebookUserId = getFacebookUserId;

  const curHost = window.location.hostname || "";
  const curPath = window.location.pathname || "";
  const curUrl = window.location.href || "";

  // 2. KHI ĐANG Ở DOMAIN FACEBOOK / META (LUỒNG XÁC THỰC, ĐĂNG NHẬP, 2FA, HOẶC CHECKPOINT):
  if (curHost.includes("facebook.com") || curHost.includes("meta.com")) {
    const isCheckpointOrAuth =
      curPath.includes("/checkpoint") ||
      curPath.includes("/two_step_verification") ||
      curPath.includes("/login") ||
      curPath.includes("/auth") ||
      curUrl.includes("checkpoint") ||
      curUrl.includes("two_step_verification") ||
      curUrl.includes("/login");

    const userId = getFacebookUserId();

    // CHỈ điều hướng về Messenger khi:
    // 1. ĐÃ XÁC THỰC THÀNH CÔNG (c_user tồn tại, userId != null)
    // 2. VÀ KHÔNG CÒN Ở MÀN HÌNH NHẬP MÃ 2FA/CHECKPOINT DỞ (đã về trang chủ fb.com hoặc xong checkpoint)
    if (userId && !isCheckpointOrAuth) {
      console.log("[DesktopBubbleChat] Phát hiện đã đăng nhập và đang ở Facebook, chuyển hướng tức thì về Messenger!");
      const accountIdx = window.__ACCOUNT_INDEX__ || 1;
      invokeTauri("report_auth_status", {
        connected: true,
        name: userId,
        accountIndex: accountIdx,
      }).catch(() => {});

      window.location.replace(TARGET_MESSENGER_URL);
      return;
    }

    // Nếu đang trong luồng xác thực (nhập mật khẩu, 2FA, quét mã QR):
    // GIỮ NGUYÊN BẢN 100% (Pass-Through) để Meta xử lý bảo mật. Không can thiệp, không chặn click, không chặn popup, không inject CSS.
    // Lắng nghe liên tục: nếu người dùng quét mã xong hoặc duyệt trên điện thoại -> URL chuyển về FB home -> tự động nạp Messenger!
    const authWatcher = setInterval(() => {
      try {
        const uid = getFacebookUserId();
        const p = window.location.pathname || "";
        const u = window.location.href || "";
        const inAuth =
          p.includes("/checkpoint") ||
          p.includes("/two_step_verification") ||
          p.includes("/login") ||
          u.includes("checkpoint");

        if (uid && !inAuth) {
          clearInterval(authWatcher);
          const accountIdx = window.__ACCOUNT_INDEX__ || 1;
          invokeTauri("report_auth_status", {
            connected: true,
            name: uid,
            accountIndex: accountIdx,
          }).catch(() => {});
          window.location.replace(TARGET_MESSENGER_URL);
        }
      } catch (e) {}
    }, 500);

    return;
  }

  // Nếu không phải messenger.com (và cũng không phải facebook/meta), không can thiệp
  if (!curHost.includes("messenger.com")) {
    return;
  }

  // 3. BẢO VỆ ĐIỀU HƯỚNG TRÊN MESSENGER.COM
  function enforceMessengerUrl() {
    try {
      const host = window.location.hostname || "";
      const href = window.location.href || "";

      if (!host || href === "about:blank") return false;

      // Nếu đã có c_user mà bị văng sang trang chủ facebook (không phải checkpoint/login)
      if (host.includes("facebook.com") && !href.includes("checkpoint") && !href.includes("/login")) {
        const uid = getFacebookUserId();
        if (uid) {
          window.location.replace(TARGET_MESSENGER_URL);
          return true;
        }
      }
    } catch (e) {}
    return false;
  }

  // 4. Chặn window.open và popup ngoài khi đang chat Messenger thông thường
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
      invokeTauri("open_external_url", { url: urlStr }).catch(() => {});
      return null;
    };
  } catch (e) {}

  // 5. Chặn click thẻ <a> chuyển hướng ra ngoài messenger.com
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
        invokeTauri("open_external_url", { url: href }).catch(() => {});
      } catch (err) {}
    },
    true
  );

  // 6. KIỂM TRA TRẠNG THÁI XÁC THỰC THỜI GIAN THỰC (BẤT BIẾN COOKIE C_USER)
  let lastReportedAuth = null;
  const AUTH_OK_TAG = " [AUTH_OK]";

  function checkAndReportAuth() {
    try {
      enforceMessengerUrl();

      const curUrl = window.location.href || "";
      const curPath = window.location.pathname || "";
      const curHost = window.location.hostname || "";

      // Nếu đã bị nhảy sang facebook.com mà đã có c_user:
      if (curHost.includes("facebook.com") || curHost.includes("meta.com")) {
        const uid = getFacebookUserId();
        const inAuth = curPath.includes("/checkpoint") || curPath.includes("/two_step_verification") || curPath.includes("/login");
        if (uid && !inAuth) {
          window.location.replace(TARGET_MESSENGER_URL);
        }
        return;
      }

      if (!curHost.includes("messenger.com")) return;

      const userId = getFacebookUserId();

      // Kiểm tra có form đăng nhập hiển thị trên DOM hay không
      const hasLoginForm = Boolean(
        document.querySelector('input[name="pass"]') ||
        document.querySelector('input[type="password"]') ||
        document.querySelector('#loginbutton') ||
        document.querySelector('button[name="login"]') ||
        document.querySelector('form#login_form') ||
        document.querySelector('form[action*="login"]')
      );

      // Trang đăng nhập rõ ràng qua URL hoặc khi có form đăng nhập trên màn hình
      const isLoginUrl =
        curPath.includes("/login") ||
        curUrl.includes("/checkpoint") ||
        curUrl.includes("two_step_verification") ||
        hasLoginForm;

      // Chat inbox paths
      const isChatPath = curPath.startsWith("/t/") || curPath.startsWith("/e2ee/");

      // Đã đăng nhập:
      // 1. Phải KHÔNG có form đăng nhập hoặc URL đăng nhập
      // 2. VÀ (Có c_user/userId hợp lệ HOẶC đang ở chat inbox path)
      const isLoggedIn =
        curHost.includes("messenger.com") &&
        !isLoginUrl &&
        (Boolean(userId) || isChatPath);

      if (isLoggedIn) {
        if (!document.title.includes(AUTH_OK_TAG)) {
          const rawTitle = (document.title || "Messenger").replace(AUTH_OK_TAG, "").trim();
          document.title = rawTitle + AUTH_OK_TAG;
        }

        // Đóng dấu UID lên URL hash để Rust backend đọc tức thì và đồng bộ 100% không phụ thuộc IPC
        if (userId && !window.location.hash.includes("auth_uid=")) {
          try {
            const newHash = "#auth_uid=" + userId;
            window.history.replaceState(null, "", newHash);
          } catch (e) {}
        }
      } else {
        // Nếu chưa đăng nhập / đang ở màn hình login: xóa dấu AUTH_OK và auth_uid nếu còn sót
        if (document.title.includes(AUTH_OK_TAG)) {
          document.title = document.title.replace(AUTH_OK_TAG, "").trim();
        }
        if (window.location.hash.includes("auth_uid=")) {
          try {
            window.history.replaceState(null, "", window.location.pathname + window.location.search);
          } catch (e) {}
        }
      }

      if (isLoggedIn !== lastReportedAuth) {
        lastReportedAuth = isLoggedIn;
        const accountIdx = window.__ACCOUNT_INDEX__ || 1;

        invokeTauri("report_auth_status", {
          connected: isLoggedIn,
          name: userId || "",
          accountIndex: accountIdx,
        }).catch(() => {});
      }
    } catch (e) {
      // Ignored
    }
  }

  window.checkAndReportAuth = checkAndReportAuth;

  // 7. Tự động trích xuất số tin nhắn chưa đọc từ document.title
  let lastBadgeCount = -1;

  function reportBadge() {
    try {
      checkAndReportAuth();

      const rawTitle = document.title || "";
      const cleanTitle = rawTitle.replace(AUTH_OK_TAG, "").trim();
      const match = cleanTitle.match(/\((\d+)\+?\)/);
      const count = match ? parseInt(match[1], 10) : 0;

      if (count !== lastBadgeCount) {
        lastBadgeCount = count;
        const accountIdx = window.__ACCOUNT_INDEX__ || 1;
        invokeTauri("update_unread_badge", { count: count, accountIndex: accountIdx }).catch(() => {});
      }

      // TẦNG 2: Bắt tiêu đề nhấp nháy thông báo tin nhắn mới từ người gửi
      const senderPattern = /^(.+?)\s+(?:đã gửi một tin nhắn|sent a message|sent a photo|đã gửi một ảnh|sent an attachment|đã gửi một file)/i;
      const titleSenderMatch = cleanTitle.match(senderPattern);
      if (titleSenderMatch && titleSenderMatch[1]) {
        const sender = titleSenderMatch[1].replace(/^\(\d+\)\s*/, "").trim();
        const accountIdx = window.__ACCOUNT_INDEX__ || 1;
        invokeTauri("report_new_message", {
          senderName: sender,
          body: "Đã gửi một tin nhắn mới",
          avatarUrl: "",
          threadId: "",
          accountIndex: accountIdx,
        }).catch(() => {});
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

  // Kiểm tra định kỳ thông minh (Throttling khi chạy ngầm để tiết kiệm CPU và RAM - Bước 5.2)
  function runAdaptiveInterval(fn, activeMs, hiddenMs) {
    let timer = null;
    function loop() {
      const ms = document.hidden ? hiddenMs : activeMs;
      timer = setTimeout(() => {
        try {
          fn();
        } catch (e) {}
        loop();
      }, ms);
    }
    document.addEventListener("visibilitychange", () => {
      clearTimeout(timer);
      loop();
    });
    loop();
  }

  runAdaptiveInterval(checkAndReportAuth, 500, 4000);
  runAdaptiveInterval(reportBadge, 800, 3000);
  runAdaptiveInterval(enforceMessengerUrl, 500, 5000);

  // 8. CƠ CHẾ FALLBACK: TRÍCH XUẤT CUỘC TRÒ CHUYỆN GẦN NHẤT AN TOÀN (BƯỚC 2.3)
  function extractRecentConversationsFallback() {
    try {
      const curHost = window.location.hostname || "";
      if (!curHost.includes("messenger.com")) return;
      const uid = getFacebookUserId();
      if (!uid) return;

      const threadLinks = document.querySelectorAll('a[href*="/t/"], a[role="link"][href*="/t/"]');
      if (!threadLinks || threadLinks.length === 0) return;

      const conversations = [];
      const seenIds = new Set();

      for (let i = 0; i < threadLinks.length && conversations.length < 10; i++) {
        const link = threadLinks[i];
        const href = link.getAttribute("href") || "";
        const match = href.match(/\/t\/([0-9a-zA-Z_-]+)/);
        if (!match) continue;

        const threadId = match[1];
        if (seenIds.has(threadId)) continue;
        seenIds.add(threadId);

        const img = link.querySelector("img");
        const avatarUrl = img && img.src && !img.src.startsWith("data:") ? img.src : "";

        const textElements = link.querySelectorAll('span[dir="auto"]');
        let name = "";
        let lastMessage = "";

        if (textElements && textElements.length > 0) {
          name = (textElements[0].textContent || "").trim();
          if (textElements.length > 1) {
            lastMessage = (textElements[1].textContent || "").trim();
          }
        }

        if (!name) {
          const aria = link.getAttribute("aria-label") || "";
          name = aria.split(",")[0] || "Người dùng Messenger";
        }

        if (name && name.length > 0 && name !== "Messenger") {
          conversations.push({
            threadId: threadId,
            senderName: name,
            body: lastMessage,
            avatarUrl: avatarUrl,
            accountIndex: window.__ACCOUNT_INDEX__ || 1,
            timestamp: Date.now(),
          });
        }
      }

      if (conversations.length > 0) {
        const accountIdx = window.__ACCOUNT_INDEX__ || 1;
        invokeTauri("sync_recent_conversations", {
          conversations: conversations,
          accountIndex: accountIdx,
        }).catch(() => {});
      }
    } catch (e) {
      // Ignored
    }
  }

  // Chạy fallback quét hội thoại sau khi DOM nạp xong và định kỳ 15s
  setTimeout(extractRecentConversationsFallback, 3500);
  setInterval(extractRecentConversationsFallback, 15000);

  // 9. Inject CSS thu gọn Messenger vừa vặn khung popup 420x620
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

  // 10. OFFLINE HANDLER & AUTO-RECONNECT (BƯỚC 5.1)
  function setupOfflineHandler() {
    function showOfflineBanner() {
      if (document.getElementById("dbc-offline-banner")) return;
      const offlineBanner = document.createElement("div");
      offlineBanner.id = "dbc-offline-banner";
      offlineBanner.style.cssText =
        "position:fixed;top:0;left:0;right:0;z-index:999999;background:#FEF2F2;border-bottom:1px solid #FCA5A5;padding:8px 12px;display:flex;align-items:center;justify-content:space-between;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;font-size:12px;color:#991B1B;box-shadow:0 2px 4px rgba(0,0,0,0.06);";
      offlineBanner.innerHTML =
        '<div style="display:flex;align-items:center;gap:6px;"><span style="font-weight:600;">⚠️ Mất kết nối Internet.</span><span>Đang chờ mạng...</span></div><button id="dbc-retry-btn" style="background:#DC2626;color:white;border:none;padding:3px 8px;border-radius:4px;font-size:11px;font-weight:600;cursor:pointer;">Thử lại</button>';

      if (document.body) {
        document.body.appendChild(offlineBanner);
        const retryBtn = document.getElementById("dbc-retry-btn");
        if (retryBtn) {
          retryBtn.onclick = () => window.location.reload();
        }
      }
    }

    function removeOfflineBanner() {
      const el = document.getElementById("dbc-offline-banner");
      if (el) el.remove();
    }

    window.addEventListener("offline", showOfflineBanner);
    window.addEventListener("online", () => {
      removeOfflineBanner();
      window.location.reload();
    });

    if (!navigator.onLine) {
      showOfflineBanner();
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", setupOfflineHandler);
  } else {
    setupOfflineHandler();
  }
})();
