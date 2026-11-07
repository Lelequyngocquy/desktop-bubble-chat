// ============================================================================
// MODULE 02: IDENTITY RECOGNITION & AUTHENTICATION ENGINE
// ============================================================================
// 1. Detects Facebook user UID via c_user cookie and Meta client runtime
// 2. Monitors authentication, 2FA, and checkpoint flows on Meta domains
// 3. Evaluates real-time login state and synchronizes with Backend via IPC report_auth_status

let lastDetectedUid = null;
let isHandlingSaveDevice = false;

/**
 * Scans Facebook/Messenger identity via cookie c_user and Meta client runtime
 * @returns {string|null} Facebook user UID or null if unauthenticated
 */
function getFacebookUserId() {
  try {
    const cookieMatch = document.cookie.match(/(?:^|;\s*)c_user=([0-9]+)/);
    if (cookieMatch && cookieMatch[1] && cookieMatch[1] !== "0") {
      const uid = cookieMatch[1];
      if (uid !== lastDetectedUid) {
        lastDetectedUid = uid;
        logDebug("AUTH-UID", `Detected UID from c_user cookie: ${uid}`, null, "info");
      }
      return uid;
    }
    if (typeof window.require === "function") {
      try {
        const currentUser = window.require("CurrentUserInitialData");
        if (currentUser && currentUser.USER_ID && currentUser.USER_ID !== "0") {
          const uid = String(currentUser.USER_ID);
          if (uid !== lastDetectedUid) {
            lastDetectedUid = uid;
            logDebug("AUTH-UID", `Detected UID from CurrentUserInitialData: ${uid}`, null, "info");
          }
          return uid;
        }
      } catch (e) {}
    }
    if (window.__user && window.__user !== "0") {
      const uid = String(window.__user);
      if (uid !== lastDetectedUid) {
        lastDetectedUid = uid;
        logDebug("AUTH-UID", `Detected UID from window.__user: ${uid}`, null, "info");
      }
      return uid;
    }
    if (window.Env && window.Env.user_id && window.Env.user_id !== "0") {
      const uid = String(window.Env.user_id);
      if (uid !== lastDetectedUid) {
        lastDetectedUid = uid;
        logDebug("AUTH-UID", `Detected UID from window.Env.user_id: ${uid}`, null, "info");
      }
      return uid;
    }
  } catch (e) {
    logDebug("AUTH-UID-ERR", `Error scanning UID: ${e}`, null, "warn");
  }
  return null;
}

window.getFacebookUserId = getFacebookUserId;

/**
 * Detects whether Meta is in an active, blocking checkpoint or 2FA verification flow
 * @returns {boolean} True if actively awaiting user verification or approval
 */
function isMetaCheckpointBlocking() {
  try {
    const path = window.location.pathname || "";
    const href = window.location.href || "";
    if (path.includes("/save-device") || path.includes("/device-based")) {
      return false;
    }
    const hasCheckpointUrl = path.includes("/checkpoint") || path.includes("/two_step_verification") || href.includes("checkpoint") || href.includes("two_step_verification") || href.includes("/security/2fac/");
    if (hasCheckpointUrl) {
      return true;
    }
    const hasCheckpointForm = Boolean(
      document.querySelector('input[name="approvals_code"]') ||
      document.querySelector('input#recovery_code_entry') ||
      document.querySelector('form[action*="checkpoint"]') ||
      document.querySelector('[data-testid="checkpoint_form"]') ||
      document.querySelector('#checkpoint_title')
    );
    if (hasCheckpointForm) {
      return true;
    }
    const bodyText = document.body ? (document.body.innerText || "") : "";
    if (bodyText.includes("Approve from another device") || bodyText.includes("Phê duyệt từ thiết bị khác") || bodyText.includes("Check your notifications") || bodyText.includes("Xác minh danh tính") || bodyText.includes("Tài khoản bị tạm khóa")) {
      return true;
    }
  } catch (e) {}
  return false;
}

/**
 * Handles device trust persistence screen (/login/save-device/) to preserve machine_id
 * @param {Function} onDone Callback invoked once device trust persistence is dispatched
 */
function handleSaveDevicePrompt(onDone) {
  if (isHandlingSaveDevice) return;
  isHandlingSaveDevice = true;
  logDebug("AUTH-DEVICE", "Handling Save Browser / Remember device prompt...", null, "info");
  try {
    const submitBtn = document.querySelector('button[value="1"]') ||
      document.querySelector('button[type="submit"]') ||
      document.querySelector('button#checkpointSubmitButton') ||
      document.querySelector('form[action*="save-device"] button') ||
      document.querySelector('form[action*="device-based"] button');
    if (submitBtn) {
      logDebug("AUTH-DEVICE", "Clicking Save Browser button to persist machine_id...", null, "info");
      submitBtn.click();
    }
  } catch (e) {
    logDebug("AUTH-DEVICE", `Error clicking submit button: ${e}`, null, "warn");
  }
  setTimeout(() => {
    isHandlingSaveDevice = false;
    if (typeof onDone === "function") {
      onDone();
    }
  }, 1200);
}

/**
 * Emits authentication completion IPC and navigates webview to TARGET_MESSENGER_URL
 * @param {string} userId Authenticated Facebook user ID
 */
function triggerMessengerHandoff(userId) {
  logDebug("AUTH-REDIRECT", `Authenticated (UID: ${userId}), handoff to Messenger...`, null, "info");
  const accountIdx = window.__ACCOUNT_INDEX__ || 1;
  invokeTauri("report_auth_status", {
    connected: true,
    name: userId,
    accountIndex: accountIdx,
  }).catch(() => {});
  window.location.replace(TARGET_MESSENGER_URL);
}

// Meta domain lifecycle watcher (Login, 2FA, Checkpoints, and Device Trust)
(function handleMetaAuthDomain() {
  const curHost = window.location.hostname || "";
  const curPath = window.location.pathname || "";
  const curUrl = window.location.href || "";
  if (curHost.includes("facebook.com") || curHost.includes("meta.com") || curHost.includes("accountscenter")) {
    logDebug("AUTH-DOMAIN", `Meta domain active: ${curHost}${curPath}`, { url: curUrl.substring(0, 100) }, "info");
    const userId = getFacebookUserId();
    const blocking = isMetaCheckpointBlocking();
    const isSaveDevice = curPath.includes("/save-device") || curPath.includes("/device-based");
    if (userId && !blocking) {
      if (isSaveDevice) {
        handleSaveDevicePrompt(() => triggerMessengerHandoff(userId));
        return;
      }
      triggerMessengerHandoff(userId);
      return;
    }
    logDebug("AUTH-DOMAIN", `Awaiting user authentication/2FA completion (${curPath})...`);
    const authWatcher = setInterval(() => {
      try {
        const uid = getFacebookUserId();
        const p = window.location.pathname || "";
        const isSave = p.includes("/save-device") || p.includes("/device-based");
        if (uid && !isMetaCheckpointBlocking()) {
          clearInterval(authWatcher);
          if (isSave) {
            handleSaveDevicePrompt(() => triggerMessengerHandoff(uid));
          } else {
            triggerMessengerHandoff(uid);
          }
        }
      } catch (e) {}
    }, 500);
  }
})();

// Real-time authentication state evaluation and IPC synchronization
let lastReportedAuth = null;

/**
 * Evaluates live authentication status and reports state to Backend Rust via IPC
 */
function checkAndReportAuth() {
  try {
    if (typeof enforceMessengerUrl === "function") {
      enforceMessengerUrl();
    }
    const curUrl = window.location.href || "";
    const curPath = window.location.pathname || "";
    const curHost = window.location.hostname || "";
    if (curHost.includes("facebook.com") || curHost.includes("meta.com") || curHost.includes("accountscenter")) {
      const uid = getFacebookUserId();
      if (uid && !isMetaCheckpointBlocking()) {
        const isSave = curPath.includes("/save-device") || curPath.includes("/device-based");
        if (isSave) {
          handleSaveDevicePrompt(() => triggerMessengerHandoff(uid));
        } else {
          triggerMessengerHandoff(uid);
        }
      }
      return;
    }
    if (!curHost.includes("messenger.com")) return;
    const userId = getFacebookUserId();
    const hasLoginForm = Boolean(
      document.querySelector('input[name="pass"]') ||
      document.querySelector('input[type="password"]') ||
      document.querySelector('#loginbutton') ||
      document.querySelector('button[name="login"]') ||
      document.querySelector('form#login_form') ||
      document.querySelector('form[action*="login"]')
    );
    const isLoginUrl =
      curPath.includes("/login") ||
      curUrl.includes("/checkpoint") ||
      curUrl.includes("two_step_verification") ||
      hasLoginForm;
    const isChatPath = curPath.startsWith("/t/") || curPath.startsWith("/e2ee/");
    const isLoggedIn =
      curHost.includes("messenger.com") &&
      !isLoginUrl &&
      (Boolean(userId) || isChatPath);
    if (isLoggedIn) {
      if (userId && !window.location.hash.includes("auth_uid=")) {
        try {
          const newHash = "#auth_uid=" + userId;
          window.history.replaceState(null, "", newHash);
          logDebug("AUTH-HASH", `Updated URL hash: ${newHash}`);
        } catch (e) {}
      }
    } else {
      if (window.location.hash.includes("auth_uid=")) {
        try {
          window.history.replaceState(null, "", window.location.pathname + window.location.search);
          logDebug("AUTH-HASH", "Cleared auth_uid from URL hash due to unauthenticated state");
        } catch (e) {}
      }
    }
    if (isLoggedIn !== lastReportedAuth) {
      logDebug("AUTH-CHANGE", `Auth status changed: ${lastReportedAuth} -> ${isLoggedIn}`, {
        userId: userId || "[none]",
        isLoginUrl,
        hasLoginForm,
        isChatPath,
        host: curHost,
        path: curPath,
      }, "info");
      lastReportedAuth = isLoggedIn;
      const accountIdx = window.__ACCOUNT_INDEX__ || 1;
      invokeTauri("report_auth_status", {
        connected: isLoggedIn,
        name: userId || "",
        accountIndex: accountIdx,
      }).then(() => {
        logDebug("AUTH-IPC", `Report auth status (${isLoggedIn}) completed`);
      }).catch((err) => {
        logDebug("AUTH-IPC", `Report auth status failed: ${err}`, null, "warn");
      });
    }
  } catch (e) {
    logDebug("AUTH-ERR", `Error in checkAndReportAuth: ${e}`, null, "error");
  }
}

window.checkAndReportAuth = checkAndReportAuth;
