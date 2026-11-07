// ============================================================================
// MODULE 03: NAVIGATION GUARD & EXTERNAL LINK INTERCEPTOR
// ============================================================================
// Nhiệm vụ:
// 1. Bảo vệ điều hướng: Luôn giữ Webview ở trang messenger.com, ngăn văng về facebook.com khi đã đăng nhập.
// 2. Chặn window.open mở cửa sổ popup lạ, điều hướng link ngoài sang trình duyệt mặc định hệ thống.
// 3. Đánh chặn sự kiện click thẻ <a> chuyển hướng ngoài domain để gọi Tauri open_external_url.

/**
 * Ensures URL remains on messenger.com if valid session exists
 */
function enforceMessengerUrl() {
  try {
    const host = window.location.hostname || "";
    const href = window.location.href || "";
    if (!host || href === "about:blank") return false;
    if (host.includes("facebook.com") && !href.includes("checkpoint") && !href.includes("/login")) {
      const uid = getFacebookUserId();
      if (uid) {
        logDebug("NAV-ENFORCE", `Redirecting back to Messenger from: ${href}`, null, "warn");
        window.location.replace(TARGET_MESSENGER_URL);
        return true;
      }
    }
  } catch (e) {
    logDebug("NAV-ERR", `Error in enforceMessengerUrl: ${e}`, null, "warn");
  }
  return false;
}

/**
 * Checks if user is authenticated and on active Messenger chat interface
 */
function isUserLoggedIn() {
  const uid = typeof getFacebookUserId === "function" ? getFacebookUserId() : null;
  const host = typeof window !== "undefined" && window.location ? (window.location.hostname || "") : "";
  const path = typeof window !== "undefined" && window.location ? (window.location.pathname || "") : "";
  const isChatPath = path.startsWith("/t/") || path.startsWith("/e2ee/");
  const isLoginUrl = path.includes("/login") || path.includes("/checkpoint") || path.includes("two_step_verification");
  const hasLoginForm = typeof document !== "undefined" && Boolean(document.querySelector('input[type="password"], form#login_form, form[action*="login"]'));
  return host.includes("messenger.com") && !isLoginUrl && !hasLoginForm && (Boolean(uid) || isChatPath);
}

/**
 * Extracts target clean URL from Meta Link Shim if present
 */
function extractDestinationUrl(rawUrl) {
  if (!rawUrl) return "";
  try {
    const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
    const parsed = new URL(rawUrl, origin);
    if ((parsed.hostname.startsWith("l.") || parsed.hostname.startsWith("lm.")) && parsed.pathname.startsWith("/l.php")) {
      const uParam = parsed.searchParams.get("u");
      if (uParam) return decodeURIComponent(uParam);
    }
    return parsed.href;
  } catch (e) {
    return String(rawUrl);
  }
}

/**
 * Determines whether URL represents internal Messenger chat routing
 */
function isInternalChatNavigation(rawUrl) {
  if (!rawUrl) return false;
  const str = String(rawUrl).trim();
  if (str.startsWith("#") || str.startsWith("javascript:")) return true;
  try {
    const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
    const parsed = new URL(str, origin);
    if (!parsed.hostname.includes("messenger.com")) return false;
    const path = parsed.pathname || "";
    return path.startsWith("/t/") || path.startsWith("/e2ee/") || path === "/new" || path === "/" || path === "";
  } catch (e) {
    return false;
  }
}

let lastOpenedUrl = "";
let lastOpenedTime = 0;

/**
 * Opens external URL safely using Tauri opener plugin or fallback command with deduplication
 */
function openUrlInDefaultBrowser(targetUrl) {
  if (!targetUrl) return;
  const cleanUrl = extractDestinationUrl(targetUrl);
  if (!cleanUrl) return;
  const now = Date.now();
  if (cleanUrl === lastOpenedUrl && now - lastOpenedTime < 1500) {
    logDebug("NAV-DEBOUNCE", `Ignoring duplicate URL open within 1500ms: ${cleanUrl}`, null, "debug");
    return;
  }
  if (now - lastOpenedTime < 500) {
    logDebug("NAV-DEBOUNCE", `Ignoring rapid burst URL open within 500ms: ${cleanUrl}`, null, "debug");
    return;
  }
  lastOpenedUrl = cleanUrl;
  lastOpenedTime = now;
  logDebug("NAV-OPEN", `Opening external URL: ${cleanUrl}`, null, "info");
  invokeTauri("plugin:opener|open_url", { url: cleanUrl }).catch(() => {
    invokeTauri("open_external_url", { url: cleanUrl }).catch((err) => {
      logDebug("NAV-OPEN", `Failed to open external URL: ${err}`, null, "warn");
    });
  });
}

// Setup external link guards and interceptors
(function setupExternalLinkGuards() {
  logDebug("NAV-INIT", "Installing navigation guards and link interceptors...");
  try {
    window.open = function (openUrl) {
      if (!openUrl) return null;
      const urlStr = String(openUrl);
      if (isUserLoggedIn()) {
        if (isInternalChatNavigation(urlStr)) {
          window.location.href = urlStr;
          return window;
        }
        openUrlInDefaultBrowser(urlStr);
        return null;
      }
      try {
        const parsed = new URL(urlStr, window.location.origin);
        if (parsed.hostname.includes("messenger.com") || parsed.hostname.includes("facebook.com") || parsed.hostname.includes("meta.com")) {
          window.location.href = urlStr;
          return window;
        }
      } catch (e) {}
      openUrlInDefaultBrowser(urlStr);
      return null;
    };
    logDebug("NAV-INIT", "window.open interceptor installed successfully");
  } catch (e) {
    logDebug("NAV-INIT", `Error installing window.open interceptor: ${e}`, null, "warn");
  }

  document.addEventListener(
    "click",
    function (e) {
      const anchor = e.target && e.target.closest ? (e.target.closest("a") || e.target.closest("[role='link']")) : null;
      if (!anchor) return;
      const rawHref = anchor.getAttribute("href") || anchor.getAttribute("data-href") || anchor.href;
      const href = typeof rawHref === "string" ? rawHref : (rawHref && rawHref.baseVal ? rawHref.baseVal : String(rawHref || ""));
      if (!href || href.startsWith("javascript:") || href.startsWith("#")) return;
      if (isUserLoggedIn()) {
        if (isInternalChatNavigation(href)) return;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        openUrlInDefaultBrowser(href);
        return;
      }
      try {
        const parsed = new URL(href, window.location.origin);
        if (parsed.hostname.includes("messenger.com") || parsed.hostname.includes("facebook.com") || parsed.hostname.includes("meta.com")) return;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        openUrlInDefaultBrowser(href);
      } catch (err) {}
    },
    true
  );
  logDebug("NAV-INIT", "Anchor click guard installed successfully");
})();

if (typeof module !== "undefined" && module.exports) {
  module.exports = { isUserLoggedIn, extractDestinationUrl, isInternalChatNavigation };
}
