// ============================================================================
// MODULE 05: NAVIGATION GUARD & EXTERNAL LINK INTERCEPTOR
// ============================================================================
// Nhiệm vụ:
// 1. Bảo vệ điều hướng: Luôn giữ Webview ở trang messenger.com, ngăn văng về facebook.com khi đã đăng nhập.
// 2. Chặn window.open mở cửa sổ popup lạ, điều hướng link ngoài sang trình duyệt mặc định hệ thống.
// 3. Đánh chặn sự kiện click thẻ <a> chuyển hướng ngoài domain để gọi Tauri open_external_url.

/**
 * Đảm bảo URL luôn ở messenger.com nếu đã có session hợp lệ
 */
function enforceMessengerUrl() {
  try {
    const host = window.location.hostname || "";
    const href = window.location.href || "";

    if (!host || href === "about:blank") return false;

    // Nếu đã có c_user mà bị văng sang trang chủ facebook (không phải checkpoint/login)
    if (host.includes("facebook.com") && !href.includes("checkpoint") && !href.includes("/login")) {
      const uid = getFacebookUserId();
      if (uid) {
        logDebug("NAV-ENFORCE", `Ngăn văng sang Facebook, điều hướng trở lại Messenger từ: ${href}`, null, "warn");
        window.location.replace(TARGET_MESSENGER_URL);
        return true;
      }
    }
  } catch (e) {
    logDebug("NAV-ERR", `Lỗi trong enforceMessengerUrl: ${e}`, null, "warn");
  }
  return false;
}

// Cài đặt bộ bảo vệ liên kết ngoài
(function setupExternalLinkGuards() {
  logDebug("NAV-INIT", "Đang cài đặt Navigation Guards & Link Interceptors...");

  // 1. Chặn window.open và popup ngoài khi đang chat Messenger thông thường
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
          logDebug("NAV-OPEN", `Mở URL nội bộ Meta/Messenger: ${urlStr}`);
          window.location.href = urlStr;
          return window;
        }
      } catch (e) {}

      // Mở link ngoài bằng trình duyệt mặc định của hệ thống
      logDebug("NAV-OPEN", `Đánh chặn window.open -> Mở qua trình duyệt ngoài: ${urlStr}`, null, "info");
      invokeTauri("open_external_url", { url: urlStr }).catch(() => {});
      return null;
    };
    logDebug("NAV-INIT", "✓ window.open Interceptor cài đặt thành công");
  } catch (e) {
    logDebug("NAV-INIT", `✕ Lỗi cài đặt window.open Interceptor: ${e}`, null, "warn");
  }

  // 2. Chặn click thẻ <a> chuyển hướng ra ngoài messenger.com
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
        logDebug("NAV-CLICK", `Đánh chặn click thẻ <a> ngoại miền -> Mở qua trình duyệt ngoài: ${href}`, null, "info");
        e.preventDefault();
        e.stopPropagation();

        // Mở ra trình duyệt ngoài
        invokeTauri("open_external_url", { url: href }).catch(() => {});
      } catch (err) {}
    },
    true
  );
  logDebug("NAV-INIT", "✓ Anchor Click Guard cài đặt thành công");
})();
