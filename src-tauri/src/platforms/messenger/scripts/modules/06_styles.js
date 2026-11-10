// ============================================================================
// MODULE 05: RESPONSIVE STYLING & COMPACT UI
// ============================================================================
// Injects custom CSS rules for compact popup chat window (404x620).

function injectMessengerStyles() {
  const styleId = "desktop-bubble-chat-style";
  if (!document.getElementById(styleId)) {
    const style = document.createElement("style");
    style.id = styleId;
    style.textContent = `
      @media (max-width: 500px) {
        div[role="complementary"] {
          display: none !important;
        }
      }
      div[role="grid"] div[role="row"], div[role="log"] > div {
        content-visibility: auto;
        contain-intrinsic-size: 0 44px;
      }
      body.app-window-unfocused * {
        animation-play-state: paused !important;
      }
    `;
    if (document.head) {
      document.head.appendChild(style);
      logDebug("STYLE-INJECT", "Injected compact responsive styles into document.head");
    } else {
      document.addEventListener("DOMContentLoaded", () => {
        if (document.head) {
          document.head.appendChild(style);
          logDebug("STYLE-INJECT", "Injected compact responsive styles on DOMContentLoaded");
        }
      });
    }
  }
}
