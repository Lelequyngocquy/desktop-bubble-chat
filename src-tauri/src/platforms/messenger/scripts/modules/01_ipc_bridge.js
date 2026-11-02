// ============================================================================
// MODULE 01: IPC BRIDGE & MESSAGE DEDUPLICATION
// ============================================================================
// Nhiệm vụ:
// 1. Cung cấp hàm logDebug toàn cục đẩy log về Rust Backend (Terminal + File) và DevTools.
// 2. Cung cấp hàm invokeTauri tương thích với cả Tauri v2 Core API và __TAURI_INTERNALS__.
// 3. Quản lý bộ đệm khử trùng lặp tin nhắn thời gian thực (sliding window 5000ms).
// 4. Cung cấp hàm reportMessageSafely để phát hiện và gửi tin nhắn mới về Rust Backend.

/**
 * Ghi log debug thống nhất:
 * 1. Đẩy về Backend Rust qua invokeTauri("log_frontend", { level, message })
 * 2. Ghi ra console của Webview để DevTools có thể xem
 * @param {string} tag Mã định danh module, vd: "IPC", "NET", "NOTIF", "AUTH", "NAV", "BADGE", "DOM", "OFFLINE", "STYLE", "LIFECYCLE"
 * @param {string} msg Thông điệp log
 * @param {any} [data] Dữ liệu đối tượng bổ trợ (tự động JSON.stringify)
 * @param {string} [level="debug"] Cấp độ log: "info" | "warn" | "error" | "debug"
 */
function logDebug(tag, msg, data, level) {
  const accountIdx = window.__ACCOUNT_INDEX__ || 1;
  const lvl = level || "debug";
  let payloadStr = "";
  if (data !== undefined && data !== null) {
    try {
      payloadStr = typeof data === "string" ? ` | Data: ${data}` : ` | Data: ${JSON.stringify(data)}`;
      if (payloadStr.length > 500) {
        payloadStr = payloadStr.substring(0, 500) + "... [truncated]";
      }
    } catch (e) {
      payloadStr = " | Data: [Unserializable]";
    }
  }

  const formattedMsg = `[M${accountIdx.toString().padStart(2, "0")}:${tag}] ${msg}${payloadStr}`;

  // 1. Console trình duyệt (DevTools)
  try {
    if (lvl === "error") {
      console.error(formattedMsg);
    } else if (lvl === "warn") {
      console.warn(formattedMsg);
    } else {
      console.log(formattedMsg);
    }
  } catch (e) {}

  // 2. Gửi về Backend Rust
  invokeTauri("log_frontend", {
    level: lvl,
    message: formattedMsg,
  }).catch(() => {});
}

/**
 * Helper IPC an toàn: Hỗ trợ cả Global Tauri v2 và __TAURI_INTERNALS__
 * @param {string} cmd Tên Tauri command
 * @param {object} args Đối số truyền vào command
 * @returns {Promise<any>}
 */
function invokeTauri(cmd, args) {
  try {
    if (
      window.__TAURI__ &&
      window.__TAURI__.core &&
      typeof window.__TAURI__.core.invoke === "function"
    ) {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    if (
      window.__TAURI_INTERNALS__ &&
      typeof window.__TAURI_INTERNALS__.invoke === "function"
    ) {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
  } catch (e) {
    // IPC không khả dụng trong ngữ cảnh hiện tại
  }
  return Promise.reject("Tauri IPC not available");
}

// Bộ đệm khử trùng lặp tin nhắn thời gian thực (5000ms sliding window)
const recentMessageDeduplicationMap = new Map();

/**
 * Báo cáo tin nhắn mới về Backend một cách an toàn và khử trùng lặp
 * @param {string} senderName Tên người gửi
 * @param {string} body Nội dung tin nhắn
 * @param {string} avatarUrl Ảnh đại diện người gửi (nếu có)
 * @param {string} threadId ID cuộc trò chuyện
 */
function reportMessageSafely(senderName, body, avatarUrl, threadId) {
  const accountIdx = window.__ACCOUNT_INDEX__ || 1;
  const cleanSender = (senderName || "Tin nhắn mới").trim();
  const cleanBody = (body || "").trim();
  const cleanThread = (threadId || "").trim();
  const key = `${accountIdx}:${cleanThread}:${cleanSender}:${cleanBody}`;
  const now = Date.now();

  logDebug("MSG-INCOMING", `Phát hiện tin nhắn từ '${cleanSender}' (thread: ${cleanThread})`, {
    sender: cleanSender,
    bodyLength: cleanBody.length,
    bodyPreview: cleanBody.substring(0, 60),
    hasAvatar: Boolean(avatarUrl),
  }, "info");

  if (recentMessageDeduplicationMap.has(key)) {
    const prevTime = recentMessageDeduplicationMap.get(key);
    if (now - prevTime < 5000) {
      logDebug("MSG-DEDUP", `Khử trùng lặp tin nhắn (bỏ qua do lặp trong 5s): key='${key}'`);
      return; // Bỏ qua trùng lặp trong 5 giây
    }
  }

  recentMessageDeduplicationMap.set(key, now);
  if (recentMessageDeduplicationMap.size > 200) {
    for (const [k, t] of recentMessageDeduplicationMap.entries()) {
      if (now - t > 10000) recentMessageDeduplicationMap.delete(k);
    }
  }

  logDebug("MSG-DISPATCH", `>>> PHÁT IPC report_new_message cho M${accountIdx}: '${cleanSender}': ${cleanBody.substring(0, 40)}`, null, "info");

  invokeTauri("report_new_message", {
    senderName: cleanSender,
    body: cleanBody,
    avatarUrl: avatarUrl || "",
    threadId: cleanThread,
    accountIndex: accountIdx,
  }).then(() => {
    logDebug("MSG-DISPATCH", `✓ report_new_message thành công cho M${accountIdx}`);
  }).catch((err) => {
    logDebug("MSG-DISPATCH", `✕ report_new_message lỗi: ${err}`, null, "error");
  });
}
