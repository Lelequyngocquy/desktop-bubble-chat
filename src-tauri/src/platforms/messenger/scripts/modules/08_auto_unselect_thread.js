// ============================================================================
// MODULE 08: AUTO-UNSELECT THREAD CONTROLLER
// ============================================================================
// Safely navigates from specific conversation (/t/<id> or /e2ee/t/<id>)
// back to Messenger chat list 5 seconds after chat window is hidden, preventing auto-seen.
// Completes background transition to chat list immediately so hover opens smoothly.

let autoUnselectTimer = null;

/**
 * Checks whether current webview is currently inside a specific conversation thread
 * @returns {boolean} True if on /t/<id> or /e2ee/t/<id>
 */
function isInChatThread() {
  try {
    if (typeof isAuthMode === "function" && isAuthMode()) return false;
    if (typeof isUserLoggedIn === "function" && !isUserLoggedIn()) return false;
    const path = (window.location && window.location.pathname) || "";
    return /^\/(?:e2ee\/)?t\/[a-zA-Z0-9._-]+/.test(path);
  } catch (e) {
    return false;
  }
}

/**
 * Finds Chats tab in navigation rail universally across all locales
 * @returns {Element|null} Chats tab element if found
 */
function findChatsTab() {
  const navTab = document.querySelector('div[role="tablist"] [role="tab"]:first-child, div[role="navigation"] a[href="/t"], div[role="navigation"] a[href="/t/"], div[role="navigation"] a[href="/"], nav a[href="/t"], nav a[href="/t/"], nav a[href="/"], a[href="/t"], a[href="/t/"]');
  if (navTab) return navTab;
  return document.querySelector('[aria-label*="Chats" i], [aria-label*="Đoạn chat" i], [aria-label*="Discussions" i], [aria-label*="Mensajes" i], [aria-label*="Nachrichten" i], [aria-label*="チャット" i], [aria-label*="聊天" i]');
}

/**
 * Ensures the chat list view is active and properly sized
 */
function activateChatList() {
  try {
    if (typeof isAuthMode === "function" && isAuthMode()) return;
    const chatsTab = findChatsTab();
    if (chatsTab && typeof chatsTab.click === "function") {
      chatsTab.click();
    }
    window.dispatchEvent(new Event("resize"));
  } catch (e) {}
}

/**
 * Ensures chat list is displayed when at Messenger home (/t)
 */
function ensureChatListVisible() {
  try {
    if (isInChatThread()) return;
    activateChatList();
  } catch (e) {}
}

/**
 * Deselects conversation via React Router (no network request) by patching history state
 * and firing a popstate event so Messenger's SPA unmounts the conversation panel.
 * Falls back to Chats tab click if popstate doesn't take effect within 200ms.
 */
function navigateToMessengerHome() {
  try {
    if (!isInChatThread()) return;
    if (typeof isAuthMode === "function" && isAuthMode()) return;
    logDebug("AUTO-UNSELECT", "SPA-navigating to /t/ via history API to clear conversation panel...", null, "info");
    history.replaceState(null, "", "/t/");
    window.dispatchEvent(new PopStateEvent("popstate", { state: null }));
    setTimeout(() => {
      if (isInChatThread() && window.__isWindowFocused !== true) {
        activateChatList();
      }
    }, 200);
  } catch (e) {
    logDebug("AUTO-UNSELECT", `Error in navigateToMessengerHome: ${e}`, null, "warn");
  }
}

/**
 * Cancels any active auto-unselect timer when chat window is active
 */
function cancelAutoUnselectCountdown() {
  if (autoUnselectTimer) {
    clearTimeout(autoUnselectTimer);
    autoUnselectTimer = null;
    logDebug("AUTO-UNSELECT", "Cancelled unselect countdown because chat window is open/focused", null, "debug");
  }
}

/**
 * Starts countdown to unselect thread after chat window is hidden
 * @param {number} delayMs Delay duration in milliseconds (default 5000)
 */
function startAutoUnselectCountdown(delayMs = 5000) {
  cancelAutoUnselectCountdown();
  if (!isInChatThread()) return;
  if (typeof window !== "undefined" && window.__isWindowFocused === true) return;
  logDebug("AUTO-UNSELECT", `Started ${delayMs}ms countdown to return to /t after hide`, null, "debug");
  autoUnselectTimer = setTimeout(() => {
    autoUnselectTimer = null;
    if (typeof window !== "undefined" && window.__isWindowFocused !== true && isInChatThread()) {
      navigateToMessengerHome();
    }
  }, delayMs);
}

if (typeof window !== "undefined") {
  window.addEventListener("blur", () => {
    startAutoUnselectCountdown(5000);
  });
  window.addEventListener("focus", () => {
    cancelAutoUnselectCountdown();
    ensureChatListVisible();
  });
  window.__cancelAutoUnselectCountdown = cancelAutoUnselectCountdown;
  window.__startAutoUnselectCountdown = startAutoUnselectCountdown;
  window.__triggerAutoUnselectIfPending = function() { startAutoUnselectCountdown(5000); };
  window.__navigateToMessengerHome = navigateToMessengerHome;
  window.__ensureChatListVisible = ensureChatListVisible;
}
