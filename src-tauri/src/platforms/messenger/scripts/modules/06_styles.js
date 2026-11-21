// ============================================================================
// MODULE 06: RESPONSIVE STYLING & COMPACT UI
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
      body {
        will-change: transform, opacity;
        transform-origin: center right;
      }
      body.dbc-chat-origin-left {
        transform-origin: center left !important;
      }
      body.dbc-chat-enter-from-right {
        animation: dbcSlideInFromRight 0.16s cubic-bezier(0.16, 1, 0.3, 1) forwards !important;
      }
      body.dbc-chat-enter-from-left {
        animation: dbcSlideInFromLeft 0.16s cubic-bezier(0.16, 1, 0.3, 1) forwards !important;
      }
      body.dbc-chat-exit-to-right {
        animation: dbcSlideOutToRight 0.10s cubic-bezier(0.4, 0, 1, 1) forwards !important;
      }
      body.dbc-chat-exit-to-left {
        animation: dbcSlideOutToLeft 0.10s cubic-bezier(0.4, 0, 1, 1) forwards !important;
      }
      @keyframes dbcSlideInFromRight {
        0% { opacity: 0; transform: translate3d(24px, 0, 0) scale(0.97); }
        100% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
      }
      @keyframes dbcSlideInFromLeft {
        0% { opacity: 0; transform: translate3d(-24px, 0, 0) scale(0.97); }
        100% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
      }
      @keyframes dbcSlideOutToRight {
        0% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
        100% { opacity: 0; transform: translate3d(18px, 0, 0) scale(0.97); }
      }
      @keyframes dbcSlideOutToLeft {
        0% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
        100% { opacity: 0; transform: translate3d(-18px, 0, 0) scale(0.97); }
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
