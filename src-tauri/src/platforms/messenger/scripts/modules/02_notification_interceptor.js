// ============================================================================
// MODULE 02: NOTIFICATION & AUDIO HOOK (PHASE 2)
// ============================================================================
// Nhiệm vụ (Giai đoạn 2):
// 1. Mock W3C Notification API & ServiceWorker showNotification.
// 2. Ép quyền thông báo luôn là 'granted'.
// 3. Khi phát hiện thông báo mới: Bóc tách payload đầy đủ { senderName, body, avatarUrl, threadId, timestamp }.
// 4. Áp dụng bộ đệm khử trùng lặp trượt (Sliding Window 3000ms).
// 5. Gửi thông tin về Backend Rust qua IPC report_incoming_notification (Zero-DOM).

(function setupNotificationInterceptor() {
  logDebug("NOTIF-INIT", "Đang cài đặt HTML5 Notification Hooks (Giai đoạn 2)...");

  // Bộ đệm khử trùng lặp trượt (Sliding Window 3000ms)
  const DEDUPE_WINDOW_MS = 3000;
  const recentSignatures = new Map(); // signature -> timestamp

  function isDuplicate(signature, now) {
    // Dọn dẹp các mốc cũ hơn 3000ms
    for (const [key, time] of recentSignatures.entries()) {
      if (now - time > DEDUPE_WINDOW_MS) {
        recentSignatures.delete(key);
      }
    }
    if (recentSignatures.has(signature)) {
      return true;
    }
    recentSignatures.set(signature, now);
    return false;
  }

  function dispatchNotificationAlert(title, options) {
    const now = Date.now();
    options = options || {};

    const rawTitle = typeof title === "string" ? title.trim() : "";
    const rawBody = typeof options.body === "string" ? options.body.trim() : "";
    const rawIcon = typeof options.icon === "string" ? options.icon : "";
    const rawTag = typeof options.tag === "string" ? options.tag : "";

    // Chữ ký định danh cho tin nhắn: title|body|tag
    const signature = `${rawTitle}|${rawBody}|${rawTag}`;

    if (isDuplicate(signature, now)) {
      logDebug("NOTIF-DEDUPE", `Bỏ qua thông báo trùng lặp trong 3000ms: '${rawTitle}'`);
      return;
    }

    const accountIdx = window.__ACCOUNT_INDEX__ || 1;
    logDebug(
      "NOTIF-ALERT",
      `>>> BẮT ĐƯỢC THÔNG BÁO MỚI CHO M${accountIdx.toString().padStart(2, "0")}! Người gửi: '${rawTitle}', Nội dung: '${rawBody}'`,
      null,
      "info"
    );

    invokeTauri("report_incoming_notification", {
      accountIndex: accountIdx,
      senderName: rawTitle,
      body: rawBody,
      avatarUrl: rawIcon,
      threadId: rawTag,
      timestamp: now,
    }).then(() => {
      logDebug("NOTIF-IPC", `✓ report_incoming_notification gửi thành công`);
    }).catch((err) => {
      logDebug("NOTIF-IPC", `✕ report_incoming_notification lỗi: ${err}`, null, "warn");
    });
  }

  // 1. Ép Notification.permission luôn là 'granted'
  try {
    Object.defineProperty(window.Notification, "permission", {
      get: function () {
        return "granted";
      },
      set: function () {},
      configurable: true,
      enumerable: true,
    });
  } catch (e) {
    try {
      window.Notification.permission = "granted";
    } catch (err) {}
  }

  // 2. Mock Notification.requestPermission()
  try {
    window.Notification.requestPermission = function (cb) {
      logDebug("NOTIF-PERM", "Messenger gọi Notification.requestPermission() -> Tự động cấp 'granted'");
      const p = Promise.resolve("granted");
      if (typeof cb === "function") cb("granted");
      return p;
    };
  } catch (e) {}

  // 3. Mock navigator.permissions.query({ name: 'notifications' }) -> 'granted'
  try {
    if (navigator.permissions && navigator.permissions.query) {
      const origQuery = navigator.permissions.query.bind(navigator.permissions);
      navigator.permissions.query = function (param) {
        if (param && param.name === "notifications") {
          logDebug("NOTIF-PERM", "navigator.permissions.query({ name: 'notifications' }) -> Giả lập 'granted'");
          return Promise.resolve({
            state: "granted",
            status: "granted",
            onchange: null,
            addEventListener: function () {},
            removeEventListener: function () {},
            dispatchEvent: function () { return false; },
          });
        }
        return origQuery(param);
      };
    }
  } catch (e) {}

  // 4. Silent Proxy cho window.Notification constructor
  try {
    function MockNotification(title, options) {
      dispatchNotificationAlert(title, options);

      this.title = title || "";
      this.body = (options && options.body) || "";
      this.icon = (options && options.icon) || "";
      this.tag = (options && options.tag) || "";
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
    }

    MockNotification.permission = "granted";
    MockNotification.requestPermission = window.Notification.requestPermission;
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
    logDebug("NOTIF-INIT", "✓ window.Notification Constructor Proxy đã kích hoạt");
  } catch (e) {
    logDebug("NOTIF-INIT", `✕ Lỗi proxy window.Notification: ${e}`, null, "warn");
  }

  // 5. Proxy ServiceWorkerRegistration.prototype.showNotification
  try {
    if (
      typeof ServiceWorkerRegistration !== "undefined" &&
      ServiceWorkerRegistration.prototype &&
      ServiceWorkerRegistration.prototype.showNotification
    ) {
      ServiceWorkerRegistration.prototype.showNotification = function (title, options) {
        logDebug("NOTIF-SW", `Bắt được ServiceWorker showNotification: '${title}'`, null, "info");
        dispatchNotificationAlert(title, options);
        return Promise.resolve();
      };
      logDebug("NOTIF-INIT", "✓ ServiceWorker showNotification Hook đã kích hoạt");
    }
  } catch (e) {
    logDebug("NOTIF-INIT", `✕ Lỗi hook ServiceWorker showNotification: ${e}`, null, "warn");
  }
})();
