// ============================================================================
// MODULE 03: NAVIGATION GUARD & EXTERNAL LINK INTERCEPTOR
// ============================================================================
// 1. Manages lifecycle routing between Auth Mode and Chat Mode
// 2. Intercepts external URLs and redirects them to default system browser
// 3. Preserves internal chat routing: /t/, /e2ee/, /media/, /stories/, blob:, data:

/**
 * Checks whether current webview window is explicitly in authentication mode
 * @returns {boolean} True if in Auth Mode
 */
function isAuthMode() {
  return Boolean(window.__IS_AUTH_MODE__);
}

/**
 * Verifies if destination URL belongs to Meta authentication and SSO domains
 * @param {string} rawUrl Destination URL
 * @returns {boolean} True if within Meta auth ecosystem
 */
function isMetaAuthDomain(rawUrl) {
  if (!rawUrl) return false;
  try {
    const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
    const parsed = new URL(rawUrl, origin);
    const h = parsed.hostname.toLowerCase();
    return (
      h.endsWith("messenger.com") ||
      h.endsWith("facebook.com") ||
      h.endsWith("meta.com") ||
      h.endsWith("fbcdn.net") ||
      h.endsWith("facebook.net") ||
      h.endsWith("workplace.com") ||
      h.includes("challenges.cloudflare.com")
    );
  } catch (e) {
    return false;
  }
}

/**
 * Ensures URL remains on messenger.com in Chat Mode, detecting session expiration
 * @returns {boolean} True if redirected back to Messenger
 */
function enforceMessengerUrl() {
  try {
    if (isAuthMode()) return false;
    const host = window.location.hostname || "";
    const href = window.location.href || "";
    if (!host || href === "about:blank") return false;
    const uid = typeof getFacebookUserId === "function" ? getFacebookUserId() : null;
    if (host.includes("facebook.com") && !href.includes("checkpoint") && !href.includes("/login")) {
      if (uid) {
        logDebug("NAV-ENFORCE", `Redirecting back to Messenger from: ${href}`, null, "warn");
        window.location.replace(TARGET_MESSENGER_URL);
        return true;
      }
    }
    if ((host.includes("facebook.com") || href.includes("/login")) && !uid) {
      logDebug("NAV-EXPIRED", `Detected unauthenticated session in chat mode: ${href}`, null, "warn");
      const accountIdx = window.__ACCOUNT_INDEX__ || 1;
      invokeTauri("report_auth_status", { connected: false, name: "", accountIndex: accountIdx }).catch(() => {});
    }
  } catch (e) {
    logDebug("NAV-ERR", `Error in enforceMessengerUrl: ${e}`, null, "warn");
  }
  return false;
}

/**
 * Checks if user is authenticated and on active Messenger chat interface
 * @returns {boolean} True if session is active
 */
function isUserLoggedIn() {
  if (isAuthMode()) return false;
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
 * @param {string} rawUrl Wrapped or direct URL
 * @returns {string} Decoded destination URL
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
 * Determines whether URL represents internal Messenger chat routing or media
 * @param {string} rawUrl Target navigation URL
 * @returns {boolean} True if safe internal chat navigation
 */
function isInternalChatNavigation(rawUrl) {
  if (!rawUrl) return false;
  const str = String(rawUrl).trim();
  if (str.startsWith("#") || str.startsWith("javascript:") || str.startsWith("blob:") || str.startsWith("data:")) return true;
  try {
    const destUrl = extractDestinationUrl(str);
    const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
    const parsed = new URL(destUrl, origin);
    if (parsed.protocol === "blob:" || parsed.protocol === "data:") return true;
    const host = parsed.hostname.toLowerCase();
    const isMessengerDomain = host === "messenger.com" || host.endsWith(".messenger.com");
    if (!isMessengerDomain) return false;
    const path = parsed.pathname || "";
    const isUnresolvedLinkShim = (host.startsWith("l.") || host.startsWith("lm.")) && path.startsWith("/l.php");
    return !isUnresolvedLinkShim;
  } catch (e) {
    return false;
  }
}

let lastOpenedUrl = "";
let lastOpenedTime = 0;

/**
 * Opens external URL safely using Tauri opener plugin with debouncing
 * @param {string} targetUrl Target destination URL
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
      if (isAuthMode()) {
        if (isMetaAuthDomain(urlStr)) {
          window.location.href = urlStr;
          return window;
        }
        openUrlInDefaultBrowser(urlStr);
        return null;
      }
      if (isUserLoggedIn()) {
        if (isInternalChatNavigation(urlStr)) {
          window.location.href = urlStr;
          return window;
        }
        openUrlInDefaultBrowser(urlStr);
        return null;
      }
      if (isMetaAuthDomain(urlStr)) {
        window.location.href = urlStr;
        return window;
      }
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
      if (isAuthMode()) {
        if (isMetaAuthDomain(href)) return;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        openUrlInDefaultBrowser(href);
        return;
      }
      if (isUserLoggedIn()) {
        if (isInternalChatNavigation(href)) return;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        openUrlInDefaultBrowser(href);
        return;
      }
      if (isMetaAuthDomain(href)) return;
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();
      openUrlInDefaultBrowser(href);
    },
    true
  );
  logDebug("NAV-INIT", "Anchor click guard installed successfully");
})();

if (typeof module !== "undefined" && module.exports) {
  module.exports = { isUserLoggedIn, extractDestinationUrl, isInternalChatNavigation, isMetaAuthDomain, isAuthMode };
}
