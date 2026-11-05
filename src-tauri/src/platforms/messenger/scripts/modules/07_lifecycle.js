// ============================================================================
// MODULE 06: LIFECYCLE SCHEDULER & BOOTSTRAP ORCHESTRATION
// ============================================================================
// Nhiệm vụ:
// 1. Cung cấp cơ chế điều phối chu kỳ thích ứng (Adaptive Interval) giảm tải CPU & RAM khi ẩn.
// 2. Lên lịch giám sát định kỳ trạng thái auth và URL (chặn chuyển hướng ngoài ý muốn).
// 3. Khởi tạo các module styles và offline handler khi DOM sẵn sàng.

/**
 * Kiểm tra định kỳ thông minh với cơ chế Throttling khi tab/webview bị ẩn
 * @param {Function} fn Hàm thực thi
 * @param {number} activeMs Khoảng thời gian khi cửa sổ đang active (ms)
 * @param {number} hiddenMs Khoảng thời gian khi cửa sổ đang ẩn/ngủ đông (ms)
 */
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
    logDebug("LIFECYCLE-VIS", `Trạng thái hiển thị webview thay đổi: document.hidden=${document.hidden}`, null, "info");
    clearTimeout(timer);
    loop();
  });
  loop();
}

// 0. Quản lý trạng thái Focus để chống Auto-Seen khi cửa sổ chat bị ẩn
if (!window.__FOCUS_INTERCEPTOR_INSTALLED__) {
  window.__FOCUS_INTERCEPTOR_INSTALLED__ = true;
  try {
    const originalHasFocus = document.hasFocus ? document.hasFocus.bind(document) : () => true;
    document.hasFocus = function () {
      if (window.__isWindowFocused === false) return false;
      return originalHasFocus();
    };
    window.addEventListener("focus", () => { window.__isWindowFocused = true; });
    window.addEventListener("blur", () => { window.__isWindowFocused = false; });
  } catch (e) {}
}

logDebug("LIFECYCLE-BOOT", ">>> BẮT ĐẦU KHỞI CHẠY HỆ THỐNG MESSENGER INJECT SCRIPTS...", {
  url: window.location.href,
  readyState: document.readyState,
}, "info");

// 1. Khởi động bộ giám sát thích ứng (Auth & URL Enforce)
runAdaptiveInterval(checkAndReportAuth, 500, 4000);
runAdaptiveInterval(enforceMessengerUrl, 500, 5000);

// 2. Khởi động các module phụ thuộc vào DOM
function bootstrapMessengerModules() {
  logDebug("LIFECYCLE-BOOT", "DOM đã sẵn sàng -> Kích hoạt Styles và OfflineHandler...");
  injectMessengerStyles();
  setupOfflineHandler();
  logDebug("LIFECYCLE-BOOT", "✓ Toàn bộ module Messenger đã kích hoạt thành công!", null, "info");
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrapMessengerModules);
} else {
  bootstrapMessengerModules();
}
