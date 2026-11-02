// ============================================================================
// MODULE 02: IDENTITY RECOGNITION & AUTHENTICATION ENGINE
// ============================================================================
// Nhiệm vụ:
// 1. Nhận diện danh tính người dùng Facebook (UID) thông qua cookie c_user và Meta Client Runtime.
// 2. Giám sát luồng xác thực / 2FA / Checkpoint trên domain facebook.com / meta.com (Pass-Through).
// 3. Đánh giá trạng thái đăng nhập thực tế (checkAndReportAuth), đóng dấu [AUTH_OK] và #auth_uid=.
// 4. Báo cáo trạng thái xác thực về Backend qua IPC report_auth_status.

let lastDetectedUid = null;

/**
 * Quét nhận diện danh tính Facebook/Messenger (Industry Standard - c_user cookie & Client Runtime)
 * @returns {string|null} UID người dùng hoặc null nếu chưa đăng nhập
 */
function getFacebookUserId() {
  try {
    // a. Cookie c_user: Tiêu chuẩn vàng của hệ thống Meta Web
    const cookieMatch = document.cookie.match(/(?:^|;\s*)c_user=([0-9]+)/);
    if (cookieMatch && cookieMatch[1] && cookieMatch[1] !== "0") {
      const uid = cookieMatch[1];
      if (uid !== lastDetectedUid) {
        lastDetectedUid = uid;
        logDebug("AUTH-UID", `Nhận diện UID từ cookie c_user: ${uid}`, null, "info");
      }
      return uid;
    }

    // b. Meta Client Module Runtime (require("CurrentUserInitialData").USER_ID)
    if (typeof window.require === "function") {
      try {
        const currentUser = window.require("CurrentUserInitialData");
        if (currentUser && currentUser.USER_ID && currentUser.USER_ID !== "0") {
          const uid = String(currentUser.USER_ID);
          if (uid !== lastDetectedUid) {
            lastDetectedUid = uid;
            logDebug("AUTH-UID", `Nhận diện UID từ CurrentUserInitialData: ${uid}`, null, "info");
          }
          return uid;
        }
      } catch (e) {}
    }

    // c. Biến ngữ cảnh toàn cục window.__user hoặc Env.user_id
    if (window.__user && window.__user !== "0") {
      const uid = String(window.__user);
      if (uid !== lastDetectedUid) {
        lastDetectedUid = uid;
        logDebug("AUTH-UID", `Nhận diện UID từ window.__user: ${uid}`, null, "info");
      }
      return uid;
    }
    if (window.Env && window.Env.user_id && window.Env.user_id !== "0") {
      const uid = String(window.Env.user_id);
      if (uid !== lastDetectedUid) {
        lastDetectedUid = uid;
        logDebug("AUTH-UID", `Nhận diện UID từ window.Env.user_id: ${uid}`, null, "info");
      }
      return uid;
    }
  } catch (e) {
    logDebug("AUTH-UID-ERR", `Lỗi quét UID: ${e}`, null, "warn");
  }
  return null;
}

window.getFacebookUserId = getFacebookUserId;

// Xử lý luồng xác thực khi đang ở domain Facebook / Meta (Login / 2FA / Checkpoint)
(function handleMetaAuthDomain() {
  const curHost = window.location.hostname || "";
  const curPath = window.location.pathname || "";
  const curUrl = window.location.href || "";

  if (curHost.includes("facebook.com") || curHost.includes("meta.com")) {
    logDebug("AUTH-DOMAIN", `Đang chạy trên domain Meta/Facebook: ${curHost}${curPath}`, { url: curUrl.substring(0, 100) }, "info");

    const isCheckpointOrAuth =
      curPath.includes("/checkpoint") ||
      curPath.includes("/two_step_verification") ||
      curPath.includes("/login") ||
      curPath.includes("/auth") ||
      curUrl.includes("checkpoint") ||
      curUrl.includes("two_step_verification") ||
      curUrl.includes("/login");

    const userId = getFacebookUserId();

    // CHỈ điều hướng về Messenger khi:
    // 1. ĐÃ XÁC THỰC THÀNH CÔNG (c_user tồn tại, userId != null)
    // 2. VÀ KHÔNG CÒN Ở MÀN HÌNH NHẬP MÃ 2FA/CHECKPOINT DỞ
    if (userId && !isCheckpointOrAuth) {
      logDebug("AUTH-REDIRECT", `Phát hiện đã đăng nhập (UID: ${userId}) trên FB, điều hướng về Messenger...`, null, "info");
      const accountIdx = window.__ACCOUNT_INDEX__ || 1;
      invokeTauri("report_auth_status", {
        connected: true,
        name: userId,
        accountIndex: accountIdx,
      }).catch(() => {});

      window.location.replace(TARGET_MESSENGER_URL);
      return;
    }

    logDebug("AUTH-DOMAIN", `Đang ở màn hình xác thực/2FA (${curPath}). Chờ người dùng hoàn tất...`);

    // Nếu đang trong luồng xác thực: Giữ nguyên bản 100% để Meta xử lý bảo mật.
    // Lắng nghe liên tục: nếu người dùng hoàn tất -> tự động nạp Messenger!
    const authWatcher = setInterval(() => {
      try {
        const uid = getFacebookUserId();
        const p = window.location.pathname || "";
        const u = window.location.href || "";
        const inAuth =
          p.includes("/checkpoint") ||
          p.includes("/two_step_verification") ||
          p.includes("/login") ||
          u.includes("checkpoint");

        if (uid && !inAuth) {
          clearInterval(authWatcher);
          logDebug("AUTH-REDIRECT", `Hoàn tất xác thực Meta 2FA (UID: ${uid}), chuyển hướng tức thì sang Messenger!`, null, "info");
          const accountIdx = window.__ACCOUNT_INDEX__ || 1;
          invokeTauri("report_auth_status", {
            connected: true,
            name: uid,
            accountIndex: accountIdx,
          }).catch(() => {});
          window.location.replace(TARGET_MESSENGER_URL);
        }
      } catch (e) {}
    }, 500);
  }
})();

// Kiểm tra trạng thái xác thực thời gian thực
let lastReportedAuth = null;
const AUTH_OK_TAG = " [AUTH_OK]";

/**
 * Đánh giá trạng thái đăng nhập thực tế của trang Messenger
 * Expose ra window.checkAndReportAuth để Backend Rust có thể gọi trực tiếp qua webview.eval()
 */
function checkAndReportAuth() {
  try {
    if (typeof enforceMessengerUrl === "function") {
      enforceMessengerUrl();
    }

    const curUrl = window.location.href || "";
    const curPath = window.location.pathname || "";
    const curHost = window.location.hostname || "";

    // Nếu đã bị nhảy sang facebook.com mà đã có c_user:
    if (curHost.includes("facebook.com") || curHost.includes("meta.com")) {
      const uid = getFacebookUserId();
      const inAuth = curPath.includes("/checkpoint") || curPath.includes("/two_step_verification") || curPath.includes("/login");
      if (uid && !inAuth) {
        logDebug("AUTH-REDIRECT", `Đang ở FB nhưng có UID=${uid}, đưa về Messenger...`);
        window.location.replace(TARGET_MESSENGER_URL);
      }
      return;
    }

    if (!curHost.includes("messenger.com")) return;

    const userId = getFacebookUserId();

    // Kiểm tra có form đăng nhập hiển thị trên DOM hay không
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

    // Chat inbox paths
    const isChatPath = curPath.startsWith("/t/") || curPath.startsWith("/e2ee/");

    // Đã đăng nhập:
    // 1. Phải KHÔNG có form đăng nhập hoặc URL đăng nhập
    // 2. VÀ (Có c_user/userId hợp lệ HOẶC đang ở chat inbox path)
    const isLoggedIn =
      curHost.includes("messenger.com") &&
      !isLoginUrl &&
      (Boolean(userId) || isChatPath);

    if (isLoggedIn) {
      if (!document.title.includes(AUTH_OK_TAG)) {
        const rawTitle = (document.title || "Messenger").replace(AUTH_OK_TAG, "").trim();
        document.title = rawTitle + AUTH_OK_TAG;
        logDebug("AUTH-TAG", `Gắn '${AUTH_OK_TAG}' vào title: '${document.title}'`);
      }

      // Đóng dấu UID lên URL hash để Rust backend đọc tức thì và đồng bộ 100% không phụ thuộc IPC
      if (userId && !window.location.hash.includes("auth_uid=")) {
        try {
          const newHash = "#auth_uid=" + userId;
          window.history.replaceState(null, "", newHash);
          logDebug("AUTH-HASH", `Cập nhật URL hash: ${newHash}`);
        } catch (e) {}
      }
    } else {
      // Nếu chưa đăng nhập / đang ở màn hình login: xóa dấu AUTH_OK và auth_uid nếu còn sót
      if (document.title.includes(AUTH_OK_TAG)) {
        document.title = document.title.replace(AUTH_OK_TAG, "").trim();
        logDebug("AUTH-TAG", `Gỡ '${AUTH_OK_TAG}' khỏi title`);
      }
      if (window.location.hash.includes("auth_uid=")) {
        try {
          window.history.replaceState(null, "", window.location.pathname + window.location.search);
          logDebug("AUTH-HASH", "Xóa auth_uid khỏi URL hash do chưa đăng nhập");
        } catch (e) {}
      }
    }

    if (isLoggedIn !== lastReportedAuth) {
      logDebug("AUTH-CHANGE", `Trạng thái đăng nhập thay đổi: ${lastReportedAuth} -> ${isLoggedIn}`, {
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
        logDebug("AUTH-IPC", `✓ Báo cáo report_auth_status (${isLoggedIn}) thành công`);
      }).catch((err) => {
        logDebug("AUTH-IPC", `✕ Báo cáo report_auth_status thất bại: ${err}`, null, "warn");
      });
    }
  } catch (e) {
    logDebug("AUTH-ERR", `Lỗi trong checkAndReportAuth: ${e}`, null, "error");
  }
}

window.checkAndReportAuth = checkAndReportAuth;
