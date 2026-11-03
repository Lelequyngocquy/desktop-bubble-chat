// ============================================================================
// MODULE 02: NOTIFICATION & AUDIO HOOK (PHASE 1)
// ============================================================================
// Nhiệm vụ (Giai đoạn 1):
// 1. Mock W3C Notification API & ServiceWorker showNotification.
// 2. Ép quyền thông báo luôn là 'granted'.
// 3. Khi phát hiện thông báo mới: Gửi cờ báo hiệu về Backend Rust qua IPC report_incoming_notification.
// 4. Tuyệt đối không can thiệp DOM hay cào nội dung tin nhắn.

(function setupNotificationInterceptor() {
  logDebug("NOTIF-INIT", "Đang cài đặt HTML5 Notification Hooks (Giai đoạn 1)...");

  let lastNotificationTime = 0;
  const NOTIF_THROTTLE_MS = 1500;

  function dispatchNotificationAlert() {
    const now = Date.now();
    if (now - lastNotificationTime < NOTIF_THROTTLE_MS) {
      logDebug("NOTIF-THROTTLE", "Bỏ qua thông báo trùng lặp trong 1500ms");
      return;
    }
    lastNotificationTime = now;

    const accountIdx = window.__ACCOUNT_INDEX__ || 1;
    logDebug("NOTIF-ALERT", `>>> BẮT ĐƯỢC THÔNG BÁO MỚI CHO M${accountIdx.toString().padStart(2, "0")}! Đang gửi IPC...`, null, "info");

    invokeTauri("report_incoming_notification", {
      accountIndex: accountIdx,
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
      dispatchNotificationAlert();

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
        dispatchNotificationAlert();
        return Promise.resolve();
      };
      logDebug("NOTIF-INIT", "✓ ServiceWorker showNotification Hook đã kích hoạt");
    }
  } catch (e) {
    logDebug("NOTIF-INIT", `✕ Lỗi hook ServiceWorker showNotification: ${e}`, null, "warn");
  }
})();
