// ============================================================================
// MODULE 09: CHAT WINDOW PIN BUTTON CONTROLLER
// ============================================================================
// Injects a lightweight floating pin button on the top-right corner of the chat window.
// Prevents auto-closing on hover leave and window blur when pinned.

const PIN_BTN_ID = "bubble-chat-pin-btn";
const PIN_STYLE_ID = "bubble-chat-pin-style";
const PIN_SVG_UNPINNED = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate(-35deg); transform-origin: center;"><line x1="12" y1="17" x2="12" y2="22"></line><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path></svg>';
const PIN_SVG_PINNED = '<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="17" x2="12" y2="22"></line><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path></svg>';

function injectPinButtonStyles() {
  if (document.getElementById(PIN_STYLE_ID)) return;
  const style = document.createElement("style");
  style.id = PIN_STYLE_ID;
  style.textContent = `
    #${PIN_BTN_ID} {
      position: fixed;
      top: 10px;
      right: 10px;
      width: 26px;
      height: 26px;
      border-radius: 9999px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: rgba(255, 255, 255, 0.75);
      backdrop-filter: blur(8px);
      -webkit-backdrop-filter: blur(8px);
      border: 1px solid rgba(0, 0, 0, 0.08);
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
      color: #65676b;
      cursor: pointer;
      z-index: 2147483647;
      opacity: 0.45;
      transition: opacity 0.2s ease, transform 0.15s ease, background 0.2s ease, color 0.2s ease, border-color 0.2s ease;
      user-select: none;
      outline: none;
      padding: 0;
      margin: 0;
    }
    #${PIN_BTN_ID}:hover {
      opacity: 1;
      background: rgba(255, 255, 255, 0.95);
      color: #050505;
      box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
      transform: scale(1.08);
    }
    #${PIN_BTN_ID}.is-pinned {
      opacity: 1;
      background: rgba(8, 102, 255, 0.12);
      border-color: rgba(8, 102, 255, 0.35);
      color: #0866FF;
      transform: scale(1);
    }
    #${PIN_BTN_ID}.is-pinned:hover {
      background: rgba(8, 102, 255, 0.2);
      transform: scale(1.08);
    }
    @media (prefers-color-scheme: dark) {
      #${PIN_BTN_ID} {
        background: rgba(36, 37, 38, 0.75);
        border-color: rgba(255, 255, 255, 0.12);
        color: #b0b3b8;
      }
      #${PIN_BTN_ID}:hover {
        background: rgba(58, 59, 60, 0.95);
        color: #e4e6eb;
      }
      #${PIN_BTN_ID}.is-pinned {
        background: rgba(8, 102, 255, 0.25);
        border-color: rgba(8, 102, 255, 0.5);
        color: #4599FF;
      }
    }
  `;
  if (document.head) document.head.appendChild(style);
  else document.addEventListener("DOMContentLoaded", () => { if (document.head) document.head.appendChild(style); });
}

function updatePinButtonUi(isPinned) {
  window.__IS_PINNED__ = !!isPinned;
  const btn = document.getElementById(PIN_BTN_ID);
  if (!btn) return;
  if (isPinned) {
    btn.classList.add("is-pinned");
    btn.title = "Bỏ ghim khung chat (Unpin)";
    btn.setAttribute("aria-label", "Unpin Chat Window");
    btn.innerHTML = PIN_SVG_PINNED;
  } else {
    btn.classList.remove("is-pinned");
    btn.title = "Ghim khung chat (Giữ luôn mở khi di chuột ra ngoài)";
    btn.setAttribute("aria-label", "Pin Chat Window");
    btn.innerHTML = PIN_SVG_UNPINNED;
  }
}

function togglePinChatState() {
  const nextPinned = !window.__IS_PINNED__;
  window.__IS_PINNED__ = nextPinned;
  updatePinButtonUi(nextPinned);
  invokeTauri("toggle_pin_chat", {
    accountIndex: window.__ACCOUNT_INDEX__ || 1,
    pinned: nextPinned,
  }).catch((err) => {
    logDebug("PIN", "Failed to toggle pin state via IPC", err, "warn");
  });
}

function ensurePinButtonInDom() {
  if (window.__IS_AUTH_MODE__ === true) {
    const existing = document.getElementById(PIN_BTN_ID);
    if (existing) existing.remove();
    return;
  }
  if (!document.body) return;
  injectPinButtonStyles();
  let btn = document.getElementById(PIN_BTN_ID);
  if (!btn) {
    btn = document.createElement("button");
    btn.id = PIN_BTN_ID;
    btn.type = "button";
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      togglePinChatState();
    });
    document.body.appendChild(btn);
    updatePinButtonUi(!!window.__IS_PINNED__);
    logDebug("PIN", "Injected pin button element into chat window DOM", null, "debug");
  }
}

function syncPinStateWithBackend() {
  invokeTauri("is_chat_pinned", {
    accountIndex: window.__ACCOUNT_INDEX__ || 1,
  }).then((isPinned) => {
    updatePinButtonUi(!!isPinned);
  }).catch(() => {});
}

function setupPinButtonLifecycle() {
  ensurePinButtonInDom();
  syncPinStateWithBackend();
  const observer = new MutationObserver(() => {
    if (!document.getElementById(PIN_BTN_ID) && document.body) {
      ensurePinButtonInDom();
    }
  });
  if (document.body) observer.observe(document.body, { childList: true });
}

if (typeof window !== "undefined") {
  window.__updatePinButtonUi = updatePinButtonUi;
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", setupPinButtonLifecycle);
  } else {
    setupPinButtonLifecycle();
  }
}
