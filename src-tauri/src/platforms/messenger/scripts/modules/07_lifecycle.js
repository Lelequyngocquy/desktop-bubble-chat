// ============================================================================
// MODULE 07: LIFECYCLE SCHEDULER & BOOTSTRAP ORCHESTRATION
// ============================================================================
// 1. Provides adaptive interval scheduling to reduce CPU and RAM usage when hidden
// 2. Periodically monitors authentication status and guards against unexpected redirects
// 3. Initializes styles and offline resilience handlers once DOM is ready

// 0. Focus state manager to prevent Auto-Seen and detect hidden window states
if (!window.__FOCUS_INTERCEPTOR_INSTALLED__) {
  window.__FOCUS_INTERCEPTOR_INSTALLED__ = true;
  try {
    const originalHasFocus = document.hasFocus ? document.hasFocus.bind(document) : () => true;
    document.hasFocus = function () {
      if (window.__isWindowFocused === false) return false;
      return originalHasFocus();
    };
    function updateFocusState(focused) {
      window.__isWindowFocused = focused;
      if (document.body) {
        document.body.classList.toggle("app-window-unfocused", !focused);
      }
    }
    window.addEventListener("focus", () => { updateFocusState(true); });
    window.addEventListener("blur", () => { updateFocusState(false); });
    document.addEventListener("visibilitychange", () => { updateFocusState(!document.hidden); });
  } catch (e) {}
}

// Smart context menu: allow text editing and selection menus while blocking generic browser menus
if (!window.__CONTEXT_MENU_GUARD_INSTALLED__) {
  window.__CONTEXT_MENU_GUARD_INSTALLED__ = true;
  document.addEventListener("contextmenu", (e) => {
    const t = e.target;
    const isEditable = t && (t.isContentEditable || (t.closest && t.closest('[contenteditable="true"]')) || t.tagName === "INPUT" || t.tagName === "TEXTAREA" || (t.closest && t.closest("input, textarea")));
    const hasSelection = Boolean(window.getSelection && window.getSelection().toString().trim());
    const isMediaOrLink = t && (t.tagName === "A" || t.tagName === "IMG" || (t.closest && t.closest("a, img")));
    if (!isEditable && !hasSelection && !isMediaOrLink) {
      e.preventDefault();
    }
  }, true);
}

// Hardware-accelerated entrance and exit animation triggers
if (!window.__CHAT_TRANSITIONS_INSTALLED__) {
  window.__CHAT_TRANSITIONS_INSTALLED__ = true;
  window.__playChatEntrance = function (direction) {
    if (!document.body) return;
    document.body.classList.remove("dbc-chat-enter-from-right", "dbc-chat-enter-from-left", "dbc-chat-exit-to-right", "dbc-chat-exit-to-left", "dbc-chat-origin-left");
    if (direction === "left") {
      document.body.classList.add("dbc-chat-origin-left");
    }
    void document.body.offsetWidth;
    const cls = direction === "left" ? "dbc-chat-enter-from-left" : "dbc-chat-enter-from-right";
    document.body.classList.add(cls);
  };
  window.__playChatExit = function (direction) {
    if (!document.body) return;
    document.body.classList.remove("dbc-chat-enter-from-right", "dbc-chat-enter-from-left", "dbc-chat-origin-left");
    if (direction === "left") {
      document.body.classList.add("dbc-chat-origin-left");
    }
    void document.body.offsetWidth;
    const cls = direction === "left" ? "dbc-chat-exit-to-left" : "dbc-chat-exit-to-right";
    document.body.classList.add(cls);
  };
}

/**
 * Smart periodic interval with adaptive throttling when tab/webview is hidden or unfocused
 * @param {Function} fn Function to execute
 * @param {number} activeMs Interval duration when window is active (ms)
 * @param {number} hiddenMs Interval duration when window is hidden or idle (ms)
 */
function runAdaptiveInterval(fn, activeMs, hiddenMs) {
  let timer = null;
  function loop() {
    const isHidden = document.hidden || window.__isWindowFocused === false;
    const ms = isHidden ? hiddenMs : activeMs;
    timer = setTimeout(() => {
      try {
        fn();
      } catch (e) {}
      loop();
    }, ms);
  }
  function handleVisibilityOrFocusChange() {
    clearTimeout(timer);
    loop();
  }
  document.addEventListener("visibilitychange", handleVisibilityOrFocusChange);
  window.addEventListener("focus", handleVisibilityOrFocusChange);
  window.addEventListener("blur", handleVisibilityOrFocusChange);
  loop();
}

logDebug("LIFECYCLE-BOOT", ">>> BẮT ĐẦU KHỞI CHẠY HỆ THỐNG MESSENGER INJECT SCRIPTS...", {
  url: window.location.href,
  readyState: document.readyState,
}, "info");

// 1. Khởi động bộ giám sát thích ứng (Auth & URL Enforce) với chu kỳ giãn cách khi ẩn (15s & 30s)
runAdaptiveInterval(checkAndReportAuth, 500, 15000);
runAdaptiveInterval(enforceMessengerUrl, 500, 30000);

// 2. Khởi động các module phụ thuộc vào DOM
function bootstrapMessengerModules() {
  logDebug("LIFECYCLE-BOOT", "DOM đã sẵn sàng -> Kích hoạt Styles và OfflineHandler...");
  injectMessengerStyles();
  setupOfflineHandler();
  logDebug("LIFECYCLE-BOOT", "✓ Toàn bộ module Messenger đã kích hoạt thành công!", null, "info");
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrapMessengerModules);
} else {
  bootstrapMessengerModules();
}
