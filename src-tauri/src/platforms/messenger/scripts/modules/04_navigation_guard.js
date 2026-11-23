// ============================================================================
// MODULE 04: NAVIGATION GUARD & EXTERNAL LINK INTERCEPTOR
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
 * Checks whether URL is a Meta outbound Link Shim (/l.php or /flx/warn)
 * @param {URL} parsed Parsed URL instance
 * @returns {boolean} True if outbound redirect shim
 */
function isMetaLinkShimUrl(parsed) {
  const h = (parsed.hostname || "").toLowerCase();
  const p = (parsed.pathname || "").toLowerCase();
  if (h === "l.messenger.com" || h === "lm.messenger.com" || h === "l.facebook.com" || h === "lm.facebook.com") return true;
  const isMetaHost = h === "messenger.com" || h.endsWith(".messenger.com") || h === "facebook.com" || h.endsWith(".facebook.com");
  return isMetaHost && (p.startsWith("/l.php") || p.startsWith("/flx/warn"));
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
    if (isMetaLinkShimUrl(parsed)) return false;
    const h = parsed.hostname.toLowerCase();
    return (
      h === "messenger.com" || h.endsWith(".messenger.com") ||
      h === "facebook.com" || h.endsWith(".facebook.com") ||
      h === "meta.com" || h.endsWith(".meta.com") ||
      h === "fbcdn.net" || h.endsWith(".fbcdn.net") ||
      h === "facebook.net" || h.endsWith(".facebook.net") ||
      h === "workplace.com" || h.endsWith(".workplace.com") ||
      h === "challenges.cloudflare.com"
    );
  } catch (e) {
    return false;
  }
}

/**
 * Verifies if destination URL is a strict Meta login, 2FA, checkpoint, or SSO path
 * @param {string} rawUrl Destination URL
 * @returns {boolean} True if strict Meta auth path
 */
function isStrictMetaAuthUrl(rawUrl) {
  if (!rawUrl) return false;
  try {
    const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
    const parsed = new URL(rawUrl, origin);
    if (isMetaLinkShimUrl(parsed)) return false;
    const h = parsed.hostname.toLowerCase();
    const p = (parsed.pathname || "").toLowerCase();
    if (h === "challenges.cloudflare.com" || h.startsWith("accountscenter.")) return true;
    if (h === "messenger.com" || h.endsWith(".messenger.com")) {
      return p.startsWith("/login") || p.includes("/checkpoint") || p.includes("two_step_verification");
    }
    if (h === "facebook.com" || h.endsWith(".facebook.com") || h === "meta.com" || h.endsWith(".meta.com")) {
      return (
        p.startsWith("/login") ||
        p.startsWith("/checkpoint") ||
        p.startsWith("/two_step_verification") ||
        p.startsWith("/security/2fac") ||
        p.startsWith("/recover") ||
        p.startsWith("/device-based") ||
        p.startsWith("/save-device") ||
        p.startsWith("/oidc") ||
        p.startsWith("/dialog/oauth") ||
        p.startsWith("/x/oauth") ||
        p.startsWith("/messenger_sync") ||
        p.startsWith("/cookie/") ||
        p.includes("/dialog/oauth") ||
        p.includes("two_step_verification")
      );
    }
    return false;
  } catch (e) {
    return false;
  }
}

/**
 * Ensures URL remains on messenger.com in Chat Mode with redirect cooldown
 * @returns {boolean} True if redirected back to Messenger
 */
function enforceMessengerUrl() {
  try {
    if (isAuthMode()) return false;
    const host = window.location.hostname || "";
    const path = window.location.pathname || "";
    const href = window.location.href || "";
    if (!host || href === "about:blank") return false;
    if (typeof isMetaSsoHandshakePath === "function" && isMetaSsoHandshakePath(host, path)) return false;
    const uid = typeof getFacebookUserId === "function" ? getFacebookUserId() : null;
    if (host.includes("facebook.com") && !href.includes("checkpoint") && !href.includes("/login")) {
      if (uid) {
        const now = Date.now();
        if (window.__DBC_LAST_REDIRECT_AT__ && now - window.__DBC_LAST_REDIRECT_AT__ < 3000) return false;
        window.__DBC_LAST_REDIRECT_AT__ = now;
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
  const hasLoginForm = typeof document !== "undefined" && Boolean(document.querySelector('input[name="pass"], form#login_form input[type="password"], form[action*="login"] input[type="password"], form#login_form, form[action*="login"], #loginbutton, button[name="login"]'));
  return host.includes("messenger.com") && !isLoginUrl && !hasLoginForm && (Boolean(uid) || isChatPath);
}

/**
 * Extracts target clean URL from up to 2 nested Meta Link Shim layers
 * @param {string} rawUrl Wrapped or direct URL
 * @returns {string} Decoded destination URL
 */
function extractDestinationUrl(rawUrl) {
  if (!rawUrl) return "";
  let current = String(rawUrl).trim();
  const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
  for (let i = 0; i < 2; i++) {
    try {
      const parsed = new URL(current, origin);
      if (!isMetaLinkShimUrl(parsed)) return parsed.href;
      const uParam = parsed.searchParams.get("u");
      if (!uParam) return parsed.href;
      const trimmed = uParam.trim();
      if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
        current = trimmed;
        continue;
      }
      const decoded = decodeURIComponent(trimmed);
      if (decoded.startsWith("http://") || decoded.startsWith("https://")) {
        current = decoded;
        continue;
      }
      return parsed.href;
    } catch (e) {
      return current;
    }
  }
  return current;
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
    const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
    const parsedRaw = new URL(str, origin);
    if (isMetaLinkShimUrl(parsedRaw)) {
      const destUrl = extractDestinationUrl(str);
      const parsedDest = new URL(destUrl, origin);
      if (isMetaLinkShimUrl(parsedDest)) return false;
      const destHost = parsedDest.hostname.toLowerCase();
      return destHost === "messenger.com" || destHost.endsWith(".messenger.com");
    }
    if (parsedRaw.protocol === "blob:" || parsedRaw.protocol === "data:") return true;
    const host = parsedRaw.hostname.toLowerCase();
    return host === "messenger.com" || host.endsWith(".messenger.com");
  } catch (e) {
    return false;
  }
}

/**
 * Checks whether hostname belongs to Meta attachment/media CDN domains
 * @param {string} host Lowercase hostname
 * @returns {boolean} True if Meta attachment CDN host
 */
function isMetaDownloadCdnHost(host) {
  return host === "fbsbx.com" || host.endsWith(".fbsbx.com") || host === "fbcdn.net" || host.endsWith(".fbcdn.net");
}

/**
 * Determines whether element or URL indicates a genuine in-app file download operation
 * @param {HTMLElement|null} anchor Target anchor if available
 * @param {string} rawUrl Navigation or download URL
 * @returns {boolean} True if genuine download intent
 */
function isDownloadIntent(anchor, rawUrl) {
  const origin = typeof window !== "undefined" && window.location ? window.location.origin : "https://www.messenger.com";
  const cleanUrl = rawUrl ? extractDestinationUrl(rawUrl) : "";
  const candidates = [rawUrl, cleanUrl].filter(Boolean);
  let hasExternalWebDestination = false;
  let hasDownloadCdnHost = false;
  let hasInternalOrBlobScheme = false;
  for (const c of candidates) {
    try {
      const parsed = new URL(c, origin);
      const proto = parsed.protocol.toLowerCase();
      if (proto === "blob:" || proto === "data:") {
        hasInternalOrBlobScheme = true;
        continue;
      }
      if (proto !== "http:" && proto !== "https:") continue;
      if (isMetaLinkShimUrl(parsed)) continue;
      const host = parsed.hostname.toLowerCase();
      const path = (parsed.pathname || "").toLowerCase();
      if (isMetaDownloadCdnHost(host)) {
        hasDownloadCdnHost = true;
        return true;
      }
      if (path.includes("file_download.php") || path.includes("attachment_preview")) return true;
      if (parsed.searchParams.get("dl") === "1" && (host === "messenger.com" || host.endsWith(".messenger.com"))) return true;
      if (host !== "messenger.com" && !host.endsWith(".messenger.com")) {
        hasExternalWebDestination = true;
      }
    } catch (e) {}
  }
  if (hasExternalWebDestination && !hasDownloadCdnHost && !hasInternalOrBlobScheme) return false;
  if (anchor && (anchor.hasAttribute("download") || anchor.getAttribute("download") !== null)) return true;
  if (anchor && (hasDownloadCdnHost || hasInternalOrBlobScheme)) {
    const aria = (anchor.getAttribute("aria-label") || "").toLowerCase();
    if (aria.includes("download") || aria.includes("tải xuống") || aria.includes("tải về")) return true;
    const text = (anchor.innerText || "").trim();
    if (/\b[^\s]+\.(txt|md|pdf|zip|rar|7z|tar|gz|doc|docx|xls|xlsx|ppt|pptx|csv|mp4|mp3|wav|png|jpg|jpeg|gif|webp)\b/i.test(text)) return true;
  }
  return false;
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
      const isDl = isDownloadIntent(null, urlStr);
      logDebug("WINDOW-OPEN", `window.open called: url='${urlStr}', isDownloadIntent=${isDl}`);
      if (isDl) {
        logDebug("WINDOW-OPEN", `Routing download URL to in-webview navigation: ${urlStr}`);
        if (typeof window.__triggerBrowserDownload === "function") {
          window.__triggerBrowserDownload(urlStr, "attachment");
        } else {
          window.location.href = urlStr;
        }
        return window;
      }
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
      if (isStrictMetaAuthUrl(urlStr)) {
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
      const target = e.target;
      const anchor = target && target.closest ? (target.closest("a") || target.closest("[role='link']")) : null;
      const button = target && target.closest ? (target.closest("button") || target.closest("[role='button']")) : null;
      const aria = (target && target.getAttribute && target.getAttribute("aria-label")) || (button && button.getAttribute && button.getAttribute("aria-label")) || (anchor && anchor.getAttribute && anchor.getAttribute("aria-label")) || "";
      const textSnippet = target && target.innerText ? target.innerText.trim().substring(0, 80) : "";
      const isFileKeyword = /\.(md|txt|pdf|zip|rar|png|jpg|mp4|docx|xlsx)/i.test(textSnippet) || /\.(md|txt|pdf|zip|rar|png|jpg|mp4|docx|xlsx)/i.test(aria);
      const isDownloadKeyword = aria.toLowerCase().includes("download") || aria.toLowerCase().includes("tải");
      if (anchor || isFileKeyword || isDownloadKeyword) {
        logDebug("CLICK-EVENT", `Click: tag=${target ? target.tagName : 'null'}, aria='${aria}', text='${textSnippet}', hasAnchor=${Boolean(anchor)}`, {
          anchorHref: anchor ? (anchor.href || anchor.getAttribute("href")) : null,
          anchorDownload: anchor ? anchor.getAttribute("download") : null,
        });
      }
      if (!anchor) return;
      const rawHref = anchor.getAttribute("href") || anchor.getAttribute("data-href") || anchor.href;
      const href = typeof rawHref === "string" ? rawHref : (rawHref && rawHref.baseVal ? rawHref.baseVal : String(rawHref || ""));
      if (!href || href.startsWith("javascript:") || href.startsWith("#")) return;
      const isDl = isDownloadIntent(anchor, href);
      const isInternal = isInternalChatNavigation(href);
      logDebug("NAV-EVAL", `Evaluating anchor: href='${href}', isDl=${isDl}, isInternal=${isInternal}`);
      if (isDl) {
        logDebug("NAV-ACTION", `Allowing native in-app download for: ${href}`);
        if (anchor.getAttribute("target") === "_blank") {
          anchor.setAttribute("target", "_self");
        }
        return;
      }
      if (isAuthMode()) {
        if (isMetaAuthDomain(href)) return;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        openUrlInDefaultBrowser(href);
        return;
      }
      if (isUserLoggedIn()) {
        if (isInternal) return;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        openUrlInDefaultBrowser(href);
        return;
      }
      if (isStrictMetaAuthUrl(href)) return;
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
  module.exports = { isUserLoggedIn, extractDestinationUrl, isInternalChatNavigation, isMetaAuthDomain, isStrictMetaAuthUrl, isDownloadIntent, isAuthMode };
}
