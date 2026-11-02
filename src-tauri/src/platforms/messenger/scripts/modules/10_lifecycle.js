// ============================================================================
// MODULE 10: LIFECYCLE SCHEDULER & BOOTSTRAP ORCHESTRATION
// ============================================================================
// Nhiệm vụ:
// 1. Cung cấp cơ chế điều phối chu kỳ thích ứng (Adaptive Interval) giảm tải CPU & RAM khi ẩn.
// 2. Lên lịch giám sát định kỳ trạng thái auth, unread badge và URL.
// 3. Khởi tạo toàn bộ các module khi DOM sẵn sàng.

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

logDebug("LIFECYCLE-BOOT", ">>> BẮT ĐẦU KHỞI CHẠY HỆ THỐNG MESSENGER INJECT SCRIPTS...", {
  url: window.location.href,
  readyState: document.readyState,
}, "info");

// 1. Khởi động bộ giám sát thích ứng
runAdaptiveInterval(checkAndReportAuth, 500, 4000);
runAdaptiveInterval(reportBadge, 800, 3000);
runAdaptiveInterval(enforceMessengerUrl, 500, 5000);

// 2. Chạy fallback quét hội thoại sau khi DOM nạp xong và định kỳ 15s
setTimeout(extractRecentConversationsFallback, 3500);
setInterval(extractRecentConversationsFallback, 15000);

// 3. Khởi động các module phụ thuộc vào DOM
function bootstrapMessengerModules() {
  logDebug("LIFECYCLE-BOOT", "DOM đã sẵn sàng -> Kích hoạt TitleObserver, Styles, và OfflineHandler...");
  initTitleObserver();
  injectMessengerStyles();
  setupOfflineHandler();
  logDebug("LIFECYCLE-BOOT", "✓ Toàn bộ module Messenger đã kích hoạt thành công!", null, "info");
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrapMessengerModules);
} else {
  bootstrapMessengerModules();
}
