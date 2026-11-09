(function () {
  const badgeId = "dbc-sample-extension-badge";
  if (document.getElementById(badgeId)) return;
  const badge = document.createElement("div");
  badge.id = badgeId;
  badge.textContent = "🧩 Sample Extension Active";
  badge.style.cssText = "position: fixed; bottom: 8px; left: 8px; z-index: 2147483647; font-size: 11px; font-weight: bold; background: #1877f2; color: #ffffff; padding: 4px 8px; border-radius: 6px; box-shadow: 0 2px 6px rgba(0,0,0,0.25); pointer-events: none; opacity: 0.9;";
  if (document.body) {
    document.body.appendChild(badge);
  } else {
    document.addEventListener("DOMContentLoaded", () => document.body && document.body.appendChild(badge));
  }
})();
