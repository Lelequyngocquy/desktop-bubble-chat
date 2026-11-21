import { beforeEach, describe, expect, it, vi } from "vitest";

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

import { formatBytes, formatSpeed } from "../updaterService";
import { feLog } from "../loggerService";
import { releaseService } from "../releaseService";
import { useUpdateStore } from "../../stores/updateStore";

describe("updater formatters", () => {
  it("formats byte sizes into KB or MB safely", () => {
    expect(formatBytes(0)).toBe("0 MB");
    expect(formatBytes(-500)).toBe("0 MB");
    expect(formatBytes(Number.NaN)).toBe("0 MB");
    expect(formatBytes(5120)).toBe("5.0 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });

  it("formats download speeds cleanly", () => {
    expect(formatSpeed(0)).toBe("0 KB/s");
    expect(formatSpeed(Number.POSITIVE_INFINITY)).toBe("0 KB/s");
    expect(formatSpeed(512 * 1024)).toBe("512.0 KB/s");
    expect(formatSpeed(2 * 1024 * 1024)).toBe("2.00 MB/s");
  });
});

describe("feLog", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("serializes errors, circular objects, and BigInt values without throwing", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const circular: Record<string, unknown> = { count: 42n };
    circular.self = circular;
    const err = new Error("ipc boom");
    feLog.error("Failed operation:", err, circular);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    const [cmd, payload] = mockInvoke.mock.calls[0];
    expect(cmd).toBe("log_frontend");
    expect(payload.level).toBe("error");
    expect(payload.message).toContain("ipc boom");
    expect(payload.message).toContain("[Circular]");
    expect(payload.message).toContain('"42"');
    consoleSpy.mockRestore();
  });
});

describe("releaseService", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    useUpdateStore.setState({
      releaseNotes: "Initial notes",
      availableVersion: "1.1.6",
      currentVersion: "1.1.5",
      releaseDate: "2026-11-20T00:00:00Z",
    });
  });

  it("fetches, normalizes, and caches GitHub releases", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => [
        { tag_name: "v1.1.6", name: "Release 1.1.6", published_at: "2026-11-20T00:00:00Z", html_url: "https://example.com/1.1.6", body: "Changelog" },
      ],
    });
    vi.stubGlobal("fetch", fetchMock);
    const first = await releaseService.fetchAllReleases({ forceRefresh: true });
    expect(first).toHaveLength(1);
    expect(first[0].tag).toBe("v1.1.6");
    const cached = await releaseService.fetchAllReleases();
    expect(cached).toHaveLength(1);
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("propagates AbortError instead of swallowing cancellation", async () => {
    const abortErr = new DOMException("Aborted", "AbortError");
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(abortErr));
    await expect(releaseService.fetchAllReleases({ forceRefresh: true })).rejects.toThrow("Aborted");
  });

  it("falls back to updateStore releaseNotes when API fails cleanly", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 503 }));
    const fallback = await releaseService.fetchAllReleases({ forceRefresh: true });
    expect(fallback).toHaveLength(1);
    expect(fallback[0].tag).toBe("v1.1.6");
    expect(fallback[0].body).toBe("Initial notes");
  });
});
