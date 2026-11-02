// ============================================================================
// MODULE 08: OFFLINE RESILIENCE & AUTO-RECONNECT
// ============================================================================
// Nhiệm vụ:
// 1. Lắng nghe trạng thái kết nối mạng của hệ thống (online / offline).
// 2. Hiển thị banner cảnh báo mất kết nối trực quan kèm nút bấm thử lại.
// 3. Tự động reload lại phiên Messenger khi mạng Internet được phục hồi.

function setupOfflineHandler() {
  logDebug("OFFLINE-INIT", "Đang cài đặt Offline/Online Event Listeners...");

  function showOfflineBanner() {
    logDebug("OFFLINE-EVENT", "Phát hiện mất kết nối mạng Internet! Hiển thị banner cảnh báo", null, "warn");
    if (document.getElementById("dbc-offline-banner")) return;
    const offlineBanner = document.createElement("div");
    offlineBanner.id = "dbc-offline-banner";
    offlineBanner.style.cssText =
      "position:fixed;top:0;left:0;right:0;z-index:999999;background:#FEF2F2;border-bottom:1px solid #FCA5A5;padding:8px 12px;display:flex;align-items:center;justify-content:space-between;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;font-size:12px;color:#991B1B;box-shadow:0 2px 4px rgba(0,0,0,0.06);";
    offlineBanner.innerHTML =
      '<div style="display:flex;align-items:center;gap:6px;"><span style="font-weight:600;">⚠️ Mất kết nối Internet.</span><span>Đang chờ mạng...</span></div><button id="dbc-retry-btn" style="background:#DC2626;color:white;border:none;padding:3px 8px;border-radius:4px;font-size:11px;font-weight:600;cursor:pointer;">Thử lại</button>';

    if (document.body) {
      document.body.appendChild(offlineBanner);
      const retryBtn = document.getElementById("dbc-retry-btn");
      if (retryBtn) {
        retryBtn.onclick = () => {
          logDebug("OFFLINE-RETRY", "Người dùng bấm nút 'Thử lại' trên offline banner, đang reload...");
          window.location.reload();
        };
      }
    }
  }

  function removeOfflineBanner() {
    logDebug("OFFLINE-EVENT", "Kết nối Internet đã phục hồi! Đang gỡ banner và reload trang...", null, "info");
    const el = document.getElementById("dbc-offline-banner");
    if (el) el.remove();
  }

  window.addEventListener("offline", showOfflineBanner);
  window.addEventListener("online", () => {
    removeOfflineBanner();
    window.location.reload();
  });

  if (!navigator.onLine) {
    showOfflineBanner();
  }
  logDebug("OFFLINE-INIT", `✓ Offline Handler đã kích hoạt (navigator.onLine=${navigator.onLine})`);
}
