// ============================================================================
// MODULE 02: NETWORK INGESTION & PROTOCOL PARSER
// ============================================================================
// Nhiệm vụ:
// 1. Monkey-patch window.fetch, XMLHttpRequest, và WebSocket để đánh chặn gói tin thời gian thực từ Meta.
// 2. Bóc tách payload GraphQL và MQTT-over-WebSocket.
// 3. Trích xuất số unread_count, tin nhắn mới (deltas), và danh sách hội thoại gần đây.
// 4. Ghi log debug chi tiết về mọi gói tin bắt được.

/**
 * Trích xuất cấu trúc dữ liệu mạng sau khi parse JSON
 * @param {object} data Dữ liệu JSON từ phản hồi mạng của Meta
 */
function handleParsedNetworkData(data) {
  if (!data || typeof data !== "object") return;
  const accountIdx = window.__ACCOUNT_INDEX__ || 1;

  try {
    // 1. Quét tìm số unread_count trực tiếp từ gói GraphQL
    function findUnreadCount(obj, depth = 0) {
      if (!obj || typeof obj !== "object" || depth > 5) return null;
      if (typeof obj.unread_count === "number") return obj.unread_count;
      if (typeof obj.num_unread_threads === "number") return obj.num_unread_threads;
      if (typeof obj.unread_threads_count === "number") return obj.unread_threads_count;
      for (const k of Object.keys(obj)) {
        if (typeof obj[k] === "object" && obj[k] !== null) {
          const found = findUnreadCount(obj[k], depth + 1);
          if (found !== null) return found;
        }
      }
      return null;
    }

    const unreadCount = findUnreadCount(data);
    if (unreadCount !== null) {
      logDebug("NET-UNREAD", `Phát hiện unread_count = ${unreadCount} từ gói mạng GraphQL`, { unreadCount }, "info");
      invokeTauri("update_unread_badge", { count: unreadCount, accountIndex: accountIdx }).catch(() => {});
    }

    // 2. Quét tìm tin nhắn mới từ Delta / Messages (MQTT Over WebSocket)
    if (Array.isArray(data.deltas)) {
      logDebug("NET-DELTA", `Phát hiện ${data.deltas.length} delta(s) trong gói mạng`, null, "info");
      for (const delta of data.deltas) {
        if (delta && delta.body && delta.messageMetadata) {
          const meta = delta.messageMetadata;
          const threadId = meta.threadKey ? (meta.threadKey.otherUserFbId || meta.threadKey.threadFbId || "") : "";
          const sender = meta.actorFbId ? `Người gửi (${meta.actorFbId})` : "Tin nhắn mới";
          logDebug("NET-DELTA-MSG", `Bóc tách tin nhắn từ delta: sender='${sender}', thread='${threadId}'`, {
            bodyPreview: delta.body.substring(0, 50),
          }, "info");
          reportMessageSafely(sender, delta.body, "", String(threadId));
        }
      }
    }

    // 3. Quét danh sách threads từ GraphQL
    function findMessageThreads(obj, depth = 0) {
      if (!obj || typeof obj !== "object" || depth > 5) return;
      if (Array.isArray(obj.nodes) && obj.nodes.length > 0 && obj.nodes[0]?.thread_key) {
        const conversations = [];
        for (const node of obj.nodes) {
          const threadId = node.thread_key?.thread_fbid || node.thread_key?.other_user_id || "";
          const lastMsg = node.last_message?.nodes?.[0];
          const senderName = node.name || lastMsg?.snippet_sender?.name || "Người dùng";
          const body = lastMsg?.snippet || node.snippet || "";
          const avatarUrl = node.customization_info?.thread_picture?.uri || "";

          if (threadId) {
            conversations.push({
              threadId: String(threadId),
              senderName: senderName,
              body: body,
              avatarUrl: avatarUrl,
              accountIndex: accountIdx,
              timestamp: Date.now(),
            });
          }
        }
        if (conversations.length > 0) {
          logDebug("NET-THREADS", `Bóc tách được ${conversations.length} cuộc hội thoại từ GraphQL node`, {
            sampleThread: conversations[0]?.senderName,
          }, "info");
          invokeTauri("sync_recent_conversations", {
            conversations: conversations,
            accountIndex: accountIdx,
          }).catch(() => {});
        }
        return;
      }

      for (const k of Object.keys(obj)) {
        if (typeof obj[k] === "object" && obj[k] !== null) {
          findMessageThreads(obj[k], depth + 1);
        }
      }
    }

    findMessageThreads(data);
  } catch (e) {
    logDebug("NET-ERR", `Lỗi phân tích JSON mạng: ${e}`, null, "warn");
  }
}

/**
 * Xử lý chuỗi thô từ response body, bóc tách tiền tố bảo mật 'for (;;);' của Meta
 * @param {string} rawText
 */
function parseNetworkPayloadString(rawText) {
  if (!rawText || typeof rawText !== "string") return;
  try {
    let clean = rawText.trim();
    if (clean.startsWith("for (;;);")) {
      clean = clean.substring(9).trim();
    }

    const lines = clean.split("\n");
    let parsedCount = 0;
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || (!trimmed.startsWith("{") && !trimmed.startsWith("["))) continue;
      try {
        const parsed = JSON.parse(trimmed);
        parsedCount++;
        handleParsedNetworkData(parsed);
      } catch (e) {}
    }
    if (parsedCount > 0) {
      logDebug("NET-PARSE", `Parse thành công ${parsedCount} đối tượng JSON từ gói phản hồi`);
    }
  } catch (e) {}
}

// Cài đặt Hooks đánh chặn Fetch & XHR & WebSocket
(function setupNetworkHooks() {
  logDebug("NET-INIT", "Đang cài đặt Network Hooks (Fetch, XHR, WebSocket)...");

  // 1. Fetch Hook
  try {
    const origFetch = window.fetch;
    window.fetch = async function (...args) {
      const response = await origFetch.apply(this, args);
      try {
        const url = typeof args[0] === "string" ? args[0] : (args[0] && args[0].url) || "";
        if (
          url.includes("/api/graphql") ||
          url.includes("/ajax/mercury") ||
          url.includes("graphql") ||
          url.includes("/chat")
        ) {
          logDebug("NET-FETCH", `Bắt request Fetch: ${url.substring(0, 120)}`);
          const clone = response.clone();
          clone.text().then((text) => {
            parseNetworkPayloadString(text);
          }).catch(() => {});
        }
      } catch (e) {}
      return response;
    };
    logDebug("NET-INIT", "✓ Fetch Hook cài đặt thành công");
  } catch (e) {
    logDebug("NET-INIT", `✕ Lỗi cài đặt Fetch Hook: ${e}`, null, "error");
  }

  // 2. XMLHttpRequest Hook
  try {
    const origXhrOpen = XMLHttpRequest.prototype.open;
    const origXhrSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function (method, url) {
      this._dbc_url = url;
      return origXhrOpen.apply(this, arguments);
    };
    XMLHttpRequest.prototype.send = function () {
      this.addEventListener("load", function () {
        try {
          const url = this._dbc_url || "";
          if (
            url.includes("/api/graphql") ||
            url.includes("/ajax/mercury") ||
            url.includes("graphql") ||
            url.includes("/chat")
          ) {
            logDebug("NET-XHR", `Bắt response XHR: ${url.substring(0, 120)} (status ${this.status})`);
            parseNetworkPayloadString(this.responseText);
          }
        } catch (e) {}
      });
      return origXhrSend.apply(this, arguments);
    };
    logDebug("NET-INIT", "✓ XHR Hook cài đặt thành công");
  } catch (e) {
    logDebug("NET-INIT", `✕ Lỗi cài đặt XHR Hook: ${e}`, null, "error");
  }

  // 3. WebSocket Traffic Hook (Real-time MQTT Messenger)
  try {
    const OrigWebSocket = window.WebSocket;
    if (OrigWebSocket) {
      window.WebSocket = function (...args) {
        const wsUrl = args[0] || "";
        logDebug("NET-WS", `Đang mở kết nối WebSocket: ${wsUrl}`, null, "info");
        const ws = new OrigWebSocket(...args);

        ws.addEventListener("open", function () {
          logDebug("NET-WS", `✓ WebSocket kết nối thành công: ${wsUrl}`, null, "info");
        });

        ws.addEventListener("message", function (event) {
          try {
            if (typeof event.data === "string") {
              logDebug("NET-WS-DATA", `Nhận frame WS Text (${event.data.length} ký tự): ${event.data.substring(0, 120)}`);
              parseNetworkPayloadString(event.data);
            } else if (event.data instanceof Blob) {
              const size = event.data.size;
              logDebug("NET-WS-BLOB", `Nhận frame WS Blob: ${size} bytes`);
              event.data.text().then((text) => {
                if (text && (text.includes("{") || text.includes("["))) {
                  logDebug("NET-WS-BLOB-TXT", `Decoded Blob text (${text.length} ký tự): ${text.substring(0, 100)}`);
                  parseNetworkPayloadString(text);
                }
              }).catch(() => {});
            } else if (event.data instanceof ArrayBuffer) {
              const size = event.data.byteLength;
              logDebug("NET-WS-BUF", `Nhận frame WS ArrayBuffer: ${size} bytes`);
              try {
                const dec = new TextDecoder("utf-8", { fatal: false });
                const text = dec.decode(event.data);
                if (text && (text.includes("{") || text.includes("["))) {
                  logDebug("NET-WS-BUF-TXT", `Decoded Buffer text (${text.length} ký tự): ${text.substring(0, 100)}`);
                  parseNetworkPayloadString(text);
                }
              } catch (e) {}
            }
          } catch (err) {}
        });

        ws.addEventListener("close", function (e) {
          logDebug("NET-WS", `WebSocket đóng: code=${e.code}, reason='${e.reason}'`, null, "warn");
        });

        return ws;
      };

      window.WebSocket.prototype = OrigWebSocket.prototype;
      window.WebSocket.CONNECTING = OrigWebSocket.CONNECTING;
      window.WebSocket.OPEN = OrigWebSocket.OPEN;
      window.WebSocket.CLOSING = OrigWebSocket.CLOSING;
      window.WebSocket.CLOSED = OrigWebSocket.CLOSED;
      logDebug("NET-INIT", "✓ WebSocket Hook cài đặt thành công");
    }
  } catch (e) {
    logDebug("NET-INIT", `✕ Lỗi cài đặt WebSocket Hook: ${e}`, null, "warn");
  }
})();
