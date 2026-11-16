// ============================================================================
// MODULE 10: IN-APP DOWNLOAD MANAGER & LIGHTBOX MEDIA INTERCEPTOR
// ============================================================================
// Intercepts file card and lightbox download button clicks to trigger native downloads

/**
 * Triggers native browser download via anchor click or blob fetch
 * @param {string} url Target resource URL
 * @param {string} suggestedName File name for download
 */
function triggerBrowserDownload(url, suggestedName) {
  if (!url) return;
  const fileName = suggestedName || "download";
  if (url.startsWith("blob:") || url.startsWith("data:")) {
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    a.style.display = "none";
    document.body.appendChild(a);
    a.click();
    setTimeout(() => { if (a.parentNode) a.parentNode.removeChild(a); }, 1000);
    return;
  }
  fetch(url)
    .then((res) => {
      if (!res.ok) throw new Error("Fetch failed: " + res.status);
      return res.blob();
    })
    .then((blob) => {
      const blobUrl = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = blobUrl;
      a.download = fileName;
      a.style.display = "none";
      document.body.appendChild(a);
      a.click();
      setTimeout(() => {
        if (a.parentNode) a.parentNode.removeChild(a);
        URL.revokeObjectURL(blobUrl);
      }, 30000);
    })
    .catch((err) => {
      logDebug("DOWNLOAD", "Fetch failed, falling back to direct anchor: " + err, null, "warn");
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      a.style.display = "none";
      document.body.appendChild(a);
      a.click();
      setTimeout(() => { if (a.parentNode) a.parentNode.removeChild(a); }, 1000);
    });
}

/**
 * Locates currently active media element and URL in lightbox
 * @returns {{type: string, url: string, suggestedName: string}|null}
 */
function getActiveLightboxMedia() {
  const dialog = document.querySelector('div[role="dialog"], [data-pagelet*="MediaViewer" i], [data-pagelet*="PhotoViewer" i]') || document;
  const video = dialog.querySelector("video");
  if (video && (video.currentSrc || video.src)) {
    const videoUrl = video.currentSrc || video.src;
    return { type: "video", url: videoUrl, suggestedName: "messenger_video_" + Date.now() + ".mp4" };
  }
  const mainImg = dialog.querySelector('img[data-visualcompletion="media-vc-image"]');
  if (mainImg && mainImg.src) {
    const ext = mainImg.src.includes(".png") ? "png" : (mainImg.src.includes(".webp") ? "webp" : "jpg");
    return { type: "image", url: mainImg.src, suggestedName: "messenger_photo_" + Date.now() + "." + ext };
  }
  const imgs = Array.from(dialog.querySelectorAll('img[src^="http"], img[src^="blob:"]'));
  if (imgs.length > 0) {
    imgs.sort((a, b) => ((b.naturalWidth || b.clientWidth || 0) * (b.naturalHeight || b.clientHeight || 0)) - ((a.naturalWidth || a.clientWidth || 0) * (a.naturalHeight || a.clientHeight || 0)));
    if (imgs[0] && imgs[0].src) {
      const ext = imgs[0].src.includes(".png") ? "png" : (imgs[0].src.includes(".webp") ? "webp" : "jpg");
      return { type: "image", url: imgs[0].src, suggestedName: "messenger_photo_" + Date.now() + "." + ext };
    }
  }
  return null;
}

// Global setup and event listener for media downloads
if (typeof window !== "undefined") {
  window.__triggerBrowserDownload = triggerBrowserDownload;
  window.__getActiveLightboxMedia = getActiveLightboxMedia;
  document.addEventListener("click", function (e) {
    const el = e.target;
    if (!el) return;
    const dlBtn = el.closest ? (el.closest('[aria-label*="Download" i]') || el.closest('[aria-label*="Tải xuống" i]') || el.closest('[aria-label*="Tải về" i]') || el.closest('[data-tooltip-content*="Download" i]') || el.closest('[data-tooltip-content*="Tải xuống" i]')) : null;
    if (dlBtn) {
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();
      const media = getActiveLightboxMedia();
      if (media && media.url) {
        logDebug("DOWNLOAD", "Triggering download for active " + media.type + ": " + media.suggestedName);
        triggerBrowserDownload(media.url, media.suggestedName);
      } else {
        logDebug("DOWNLOAD", "No active media element located in lightbox", null, "warn");
      }
      return;
    }
    const text = el.innerText || (el.getAttribute && el.getAttribute("aria-label")) || "";
    const fileMatch = text.match(/\b([a-zA-Z0-9_\-\.\s]+\.(txt|md|pdf|zip|rar|docx|xlsx|csv|mp4|mp3|png|jpg))\b/i);
    if (fileMatch) {
      const parentAnchor = el.closest ? el.closest("a") : null;
      if (!parentAnchor) {
        const card = el.closest ? (el.closest('[role="button"]') || el.closest('[data-testid*="file"]')) : null;
        const innerA = card ? card.querySelector("a[href]") : null;
        if (innerA) {
          innerA.click();
        }
      }
    }
  }, true);
}
