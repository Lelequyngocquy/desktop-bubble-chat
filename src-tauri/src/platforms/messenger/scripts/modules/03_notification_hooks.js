// ============================================================================
// MODULE 03: NOTIFICATION & AUDIO INTERCEPTION
// ============================================================================
// Nhiệm vụ:
// 1. Giả lập W3C Notification API (Non-DOM Interception) để bắt tin nhắn mới khi Meta kích hoạt thông báo trình duyệt.
// 2. Mock ServiceWorkerRegistration.prototype.showNotification.
// 3. Đánh chặn sự kiện phát âm thanh (Audio Chime Hook) của HTMLMediaElement và Web Audio API.

(function setupNotificationHooks() {
  logDebug("NOTIF-INIT", "Đang cài đặt Notification & Audio Hooks...");

  // 1. TẦNG NON-DOM INTERCEPTION VIA W3C NOTIFICATION API (FALLBACK CAO CẤP)
  try {
    function MockNotification(title, options) {
      options = options || {};

      logDebug("NOTIF-CATCH", `>>> BẮT ĐƯỢC THÔNG BÁO W3C: '${title}'`, {
        body: options.body,
        icon: options.icon ? options.icon.substring(0, 80) : "",
        tag: options.tag,
      }, "info");

      // Báo cáo tin nhắn qua hàm khử trùng an toàn
      reportMessageSafely(
        title || "Tin nhắn mới",
        options.body || "",
        options.icon || "",
        options.tag || ""
      );

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
      logDebug("NOTIF-PERM", "Messenger gọi Notification.requestPermission() -> Tự động cấp quyền 'granted'", null, "info");
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
        logDebug("NOTIF-SW", `Bắt được ServiceWorker showNotification: '${title}'`, options, "info");
        new MockNotification(title, options);
        return Promise.resolve();
      };
    }

    // Mock navigator.permissions.query({ name: 'notifications' }) -> 'granted'
    if (navigator.permissions && navigator.permissions.query) {
      const origPermQuery = navigator.permissions.query.bind(navigator.permissions);
      navigator.permissions.query = function (param) {
        if (param && param.name === "notifications") {
          logDebug("NOTIF-PERM", "navigator.permissions.query({ name: 'notifications' }) -> Giả lập cấp quyền 'granted'", null, "info");
          return Promise.resolve({
            state: "granted",
            status: "granted",
            onchange: null,
            addEventListener: function () {},
            removeEventListener: function () {},
            dispatchEvent: function () { return false; },
          });
        }
        return origPermQuery(param);
      };
    }

    logDebug("NOTIF-INIT", "✓ W3C Notification Mock & Permissions Query cài đặt thành công");
  } catch (e) {
    logDebug("NOTIF-INIT", `✕ Lỗi cài đặt Notification interceptor: ${e}`, null, "warn");
  }

  // 2. AUDIO CHIME HOOK (HTMLMediaElement + AudioConstructor)
  try {
    const origAudioPlay = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      try {
        const src = this.src || this.currentSrc || "";
        const isAudio = this instanceof HTMLAudioElement || this.tagName === "AUDIO";
        logDebug("AUDIO-PLAY", `>>> PHÁT ÂM THANH HTMLMediaElement (tag: ${this.tagName}, src: ${src ? src.substring(0, 100) : "[empty]"})`, {
          isAudio,
          duration: this.duration,
          currentTime: this.currentTime,
        }, "info");
      } catch (err) {}
      return origAudioPlay.apply(this, arguments);
    };
    logDebug("NOTIF-INIT", "✓ HTMLMediaElement Audio Play Hook cài đặt thành công");
  } catch (e) {
    logDebug("NOTIF-INIT", `✕ Lỗi cài đặt Audio Play Hook: ${e}`, null, "warn");
  }

  // 3. WEB AUDIO API HOOK (AudioContext.createBufferSource & decodeAudioData)
  try {
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    if (AudioCtx && AudioCtx.prototype) {
      const origCreateBufferSource = AudioCtx.prototype.createBufferSource;
      AudioCtx.prototype.createBufferSource = function () {
        const source = origCreateBufferSource.apply(this, arguments);
        const origStart = source.start;
        const ctx = this;
        source.start = function (...args) {
          logDebug("AUDIO-WEBAUDIO", `>>> PHÁT ÂM THANH QUA Web Audio API (AudioBufferSourceNode.start, ctx.state='${ctx.state}')`, null, "info");
          // Nếu AudioContext bị suspended do chính sách Autoplay của WebView2, tự động resume
          if (ctx.state === "suspended" && typeof ctx.resume === "function") {
            ctx.resume().catch(() => {});
          }
          return origStart.apply(this, args);
        };
        return source;
      };

      const origDecodeAudioData = AudioCtx.prototype.decodeAudioData;
      if (origDecodeAudioData) {
        AudioCtx.prototype.decodeAudioData = function (audioData, successCallback, errorCallback) {
          const byteLen = audioData ? audioData.byteLength || 0 : 0;
          logDebug("AUDIO-DECODE", `Meta giải mã AudioBuffer qua decodeAudioData (${byteLen} bytes)`);
          return origDecodeAudioData.apply(this, arguments);
        };
      }

      logDebug("NOTIF-INIT", "✓ Web Audio API Hook cài đặt thành công");
    }
  } catch (e) {
    logDebug("NOTIF-INIT", `✕ Lỗi cài đặt Web Audio API Hook: ${e}`, null, "warn");
  }
})();
