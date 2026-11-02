// ============================================================================
// MODULE 01: IPC BRIDGE & LOGGING ENGINE
// ============================================================================
// Nhiệm vụ:
// 1. Cung cấp hàm logDebug toàn cục đẩy log về Rust Backend (Terminal + File) và DevTools.
// 2. Cung cấp hàm invokeTauri tương thích với cả Tauri v2 Core API và __TAURI_INTERNALS__.

/**
 * Ghi log debug thống nhất:
 * 1. Đẩy về Backend Rust qua invokeTauri("log_frontend", { level, message })
 * 2. Ghi ra console của Webview để DevTools có thể xem
 * @param {string} tag Mã định danh module, vd: "IPC", "AUTH", "NAV", "OFFLINE", "STYLE", "LIFECYCLE"
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
