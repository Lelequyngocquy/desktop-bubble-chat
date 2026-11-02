// ============================================================================
// MODULE 07: DOM FALLBACK CONVERSATION SCRAPER
// ============================================================================
// Nhiệm vụ:
// 1. Quét danh sách cuộc trò chuyện gần nhất từ DOM (Fallback khi network hook bị chặn hoặc chưa có event).
// 2. Trích xuất threadId, tên người gửi, đoạn trích tin nhắn cuối và avatar URL.
// 3. Đồng bộ danh sách hội thoại về Backend qua IPC sync_recent_conversations.

let lastScrapedConversationsHash = "";

/**
 * Trích xuất các cuộc trò chuyện gần nhất từ DOM giao diện Messenger
 */
function extractRecentConversationsFallback() {
  try {
    const curHost = window.location.hostname || "";
    if (!curHost.includes("messenger.com")) return;
    const uid = getFacebookUserId();
    if (!uid) {
      logDebug("DOM-SCRAPE", "Bỏ qua cào DOM do chưa nhận diện được UID đăng nhập");
      return;
    }

    const threadLinks = document.querySelectorAll('a[href*="/t/"], a[role="link"][href*="/t/"]');
    if (!threadLinks || threadLinks.length === 0) {
      logDebug("DOM-SCRAPE", "Không tìm thấy thẻ link hội thoại 'a[href*=\"/t/\"]' trong DOM hiện tại");
      return;
    }

    logDebug("DOM-SCRAPE", `Tìm thấy ${threadLinks.length} thẻ link cuộc trò chuyện trên giao diện`);

    const conversations = [];
    const seenIds = new Set();

    for (let i = 0; i < threadLinks.length && conversations.length < 10; i++) {
      const link = threadLinks[i];
      const href = link.getAttribute("href") || "";
      const match = href.match(/\/t\/([0-9a-zA-Z_-]+)/);
      if (!match) continue;

      const threadId = match[1];
      if (seenIds.has(threadId)) continue;
      seenIds.add(threadId);

      const img = link.querySelector("img");
      const avatarUrl = img && img.src && !img.src.startsWith("data:") ? img.src : "";

      const textElements = link.querySelectorAll('span[dir="auto"]');
      let name = "";
      let lastMessage = "";

      if (textElements && textElements.length > 0) {
        name = (textElements[0].textContent || "").trim();
        if (textElements.length > 1) {
          lastMessage = (textElements[1].textContent || "").trim();
        }
      }

      if (!name) {
        const aria = link.getAttribute("aria-label") || "";
        name = aria.split(",")[0] || "Người dùng Messenger";
      }

      if (name && name.length > 0 && name !== "Messenger") {
        conversations.push({
          threadId: threadId,
          senderName: name,
          body: lastMessage,
          avatarUrl: avatarUrl,
          accountIndex: window.__ACCOUNT_INDEX__ || 1,
          timestamp: Date.now(),
        });
      }
    }

    if (conversations.length > 0) {
      const simpleHash = conversations.map((c) => `${c.threadId}:${c.body}`).join("|");
      if (simpleHash !== lastScrapedConversationsHash) {
        lastScrapedConversationsHash = simpleHash;
        logDebug("DOM-SCRAPE", `Trích xuất thành công ${conversations.length} cuộc hội thoại từ DOM`, {
          threads: conversations.slice(0, 3).map((c) => ({ name: c.senderName, snippet: c.body })),
        }, "info");

        const accountIdx = window.__ACCOUNT_INDEX__ || 1;
        invokeTauri("sync_recent_conversations", {
          conversations: conversations,
          accountIndex: accountIdx,
        }).then(() => {
          logDebug("DOM-IPC", `✓ sync_recent_conversations (${conversations.length} threads) gửi thành công`);
        }).catch((err) => {
          logDebug("DOM-IPC", `✕ sync_recent_conversations thất bại: ${err}`, null, "warn");
        });
      }
    }
  } catch (e) {
    logDebug("DOM-ERR", `Lỗi cào dữ liệu hội thoại từ DOM: ${e}`, null, "warn");
  }
}
