// ============================================================================
// MODULE 06: UNREAD BADGE & TITLE OBSERVER
// ============================================================================
// Nhiệm vụ:
// 1. Theo dõi thẻ <title> thông qua MutationObserver để bắt số tin nhắn chưa đọc dạng (N).
// 2. Phát hiện tiêu đề nhấp nháy khi có tin nhắn mới ("Người gửi đã gửi một tin nhắn...").
// 3. Báo cáo số badge chưa đọc về Backend qua IPC update_unread_badge.

let lastBadgeCount = -1;
let lastSeenRawTitle = "";

/**
 * Trích xuất số tin nhắn chưa đọc và tên người gửi từ document.title
 */
function reportBadge() {
  try {
    if (typeof checkAndReportAuth === "function") {
      checkAndReportAuth();
    }

    const rawTitle = document.title || "";
    if (rawTitle !== lastSeenRawTitle) {
      lastSeenRawTitle = rawTitle;
      logDebug("BADGE-TITLE", `document.title ghi nhận mới: '${rawTitle}'`);
    }

    const cleanTitle = rawTitle.replace(AUTH_OK_TAG, "").trim();
    const match = cleanTitle.match(/\((\d+)\+?\)/);
    const count = match ? parseInt(match[1], 10) : 0;

    if (count !== lastBadgeCount) {
      logDebug("BADGE-COUNT", `Thay đổi số badge chưa đọc: ${lastBadgeCount} -> ${count} (từ title '${cleanTitle}')`, { count }, "info");
      lastBadgeCount = count;
      const accountIdx = window.__ACCOUNT_INDEX__ || 1;
      invokeTauri("update_unread_badge", { count: count, accountIndex: accountIdx }).then(() => {
        logDebug("BADGE-IPC", `✓ update_unread_badge (${count}) gửi thành công`);
      }).catch((err) => {
        logDebug("BADGE-IPC", `✕ update_unread_badge (${count}) lỗi: ${err}`, null, "warn");
      });
    }

    // TẦNG 2: Bắt tiêu đề nhấp nháy thông báo tin nhắn mới từ người gửi (Fallback)
    // Các biến thể thông báo tin nhắn đa ngôn ngữ của Facebook Messenger:
    const senderPattern = /^(.+?)\s+(?:đã gửi một tin nhắn|đã gửi một ảnh|đã gửi một file|đã gửi một nhãn dán|đã gửi một video|đã gửi một đoạn ghi âm|sent a message|sent a photo|sent an attachment|sent a file|sent a sticker|sent a video|sent an audio)/i;
    const titleSenderMatch = cleanTitle.match(senderPattern);
    if (titleSenderMatch && titleSenderMatch[1]) {
      const sender = titleSenderMatch[1].replace(/^\(\d+\)\s*/, "").trim();
      logDebug("BADGE-FLASH", `>>> BẮT ĐƯỢC TIN NHẮN TỪ TIÊU ĐỀ NHẤP NHÁY: '${sender}'`, { rawTitle }, "info");
      reportMessageSafely(sender, "Đã gửi một tin nhắn mới", "", "");
    }
  } catch (e) {
    logDebug("BADGE-ERR", `Lỗi trong reportBadge: ${e}`, null, "warn");
  }
}

// Lắng nghe thẻ <title> thay đổi nội dung qua MutationObserver
const titleObserver = new MutationObserver(() => {
  reportBadge();
});

function initTitleObserver() {
  const titleEl = document.querySelector("title");
  if (titleEl) {
    titleObserver.observe(titleEl, { childList: true, characterData: true, subtree: true });
    logDebug("BADGE-INIT", `✓ Title MutationObserver đã gắn kết vào thẻ <title> (tiêu đề hiện tại: '${document.title}')`);
  } else {
    logDebug("BADGE-INIT", "Chưa tìm thấy thẻ <title>, hẹn giờ thử lại sau 500ms...");
    setTimeout(initTitleObserver, 500);
  }
  reportBadge();
}
