// ============================================================================
// MODULE 10: IN-APP DOWNLOAD MANAGER & E2EE BLOB STREAMER
// ============================================================================
// Handles blob: URL downloads from E2EE chats and manages chunked IPC transfer

/**
 * Converts ArrayBuffer to Base64 string safely without stack overflow
 * @param {ArrayBuffer} buffer Data buffer
 * @returns {string} Base64 string
 */
function bufferToBase64(buffer) {
  let binary = "";
  const bytes = new Uint8Array(buffer);
  const chunkSize = 8192;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const sub = bytes.subarray(i, i + chunkSize);
    binary += String.fromCharCode.apply(null, sub);
  }
  return window.btoa(binary);
}

/**
 * Handles in-app download for blob: URLs (used in E2EE encrypted chats)
 * @param {string} blobUrl Destination blob URL
 * @param {string} suggestedName Proposed file name
 */
async function handleBlobDownload(blobUrl, suggestedName) {
  try {
    const accountIdx = window.__ACCOUNT_INDEX__ || 1;
    const windowLabel = `chat-messenger-${String(accountIdx).padStart(2, "0")}`;
    logDebug("DOWNLOAD", `Prompting save dialog for blob: ${suggestedName}`);
    const targetPath = await invokeTauri("prompt_save_file", {
      windowLabel,
      defaultName: suggestedName || "attachment",
    });
    if (!targetPath) {
      logDebug("DOWNLOAD", "User cancelled save file dialog");
      return;
    }
    logDebug("DOWNLOAD", `Fetching blob data: ${blobUrl}`);
    const res = await fetch(blobUrl);
    const fullBuffer = await res.arrayBuffer();
    const totalBytes = fullBuffer.byteLength;
    logDebug("DOWNLOAD", `Blob fetched successfully, total size: ${totalBytes} bytes`);

    // Stream in chunks (512 KB per chunk) to avoid IPC memory pressure
    const CHUNK_SIZE = 512 * 1024;
    let offset = 0;
    let isFirst = true;
    while (offset < totalBytes) {
      const slice = fullBuffer.slice(offset, offset + CHUNK_SIZE);
      const chunkBase64 = bufferToBase64(slice);
      await invokeTauri("save_blob_chunk", {
        filePath: targetPath,
        chunkBase64,
        isFirstChunk: isFirst,
      });
      isFirst = false;
      offset += CHUNK_SIZE;
    }

    logDebug("DOWNLOAD", `Successfully saved blob to: ${targetPath}`);
    const fileName = suggestedName || targetPath.split(/[/\\]/).pop() || "file";
    await invokeTauri("notify_download_complete", {
      fileName,
      filePath: targetPath,
    });
  } catch (err) {
    logDebug("DOWNLOAD", `Failed to handle blob download: ${err}`, null, "error");
  }
}

// Attach globally
if (typeof window !== "undefined") {
  window.__handleBlobDownload = handleBlobDownload;
}
