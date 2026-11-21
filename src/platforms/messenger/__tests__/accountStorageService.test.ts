import { beforeEach, describe, expect, it, vi } from "vitest";
import { accountStorageService } from "../services/accountStorageService";

function createMemoryStorage() {
  const map = new Map<string, string>();
  return {
    getItem: (key: string) => map.get(key) ?? null,
    setItem: (key: string, value: string) => {
      map.set(key, value);
    },
    removeItem: (key: string) => {
      map.delete(key);
    },
    clear: () => map.clear(),
  };
}

describe("accountStorageService", () => {
  let memoryStorage: ReturnType<typeof createMemoryStorage>;

  beforeEach(() => {
    memoryStorage = createMemoryStorage();
    Object.defineProperty(globalThis, "localStorage", {
      value: memoryStorage,
      configurable: true,
      writable: true,
    });
  });

  it("returns default account 01 when storage is empty or corrupted", () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    expect(accountStorageService.load().map((a) => a.index)).toEqual([1]);
    memoryStorage.setItem("desktop_bubble_chat_messenger_accounts", "{invalid-json");
    expect(accountStorageService.load().map((a) => a.index)).toEqual([1]);
    warnSpy.mockRestore();
  });

  it("sanitizes invalid entries, deduplicates indices, and sorts ascending", () => {
    memoryStorage.setItem(
      "desktop_bubble_chat_messenger_accounts",
      JSON.stringify([
        { id: "bad-0", index: 0, isConnected: true },
        { id: "bad-9", index: 9, isConnected: true },
        { id: "messenger-03", index: 3, isConnected: true, shouldRestoreBubble: true, soundEnabled: false },
        { id: "dup-03", index: 3, isConnected: false },
        { id: "messenger-01", index: 1, isConnected: true, shouldRestoreBubble: false, isAuthenticating: true },
      ]),
    );
    const loaded = accountStorageService.load();
    expect(loaded.map((a) => a.index)).toEqual([1, 3]);
    expect(loaded[0].isAuthenticating).toBe(false);
    expect(loaded[0].lifecycleState).toBe("cold");
    expect(loaded[1].lifecycleState).toBe("active");
    expect(loaded[1].soundEnabled).toBe(false);
  });

  it("persists normalized fields and clears storage cleanly", () => {
    const defaults = accountStorageService.load();
    accountStorageService.save(defaults);
    expect(memoryStorage.getItem("desktop_bubble_chat_messenger_accounts")).toContain("messenger-01");
    accountStorageService.clear();
    expect(memoryStorage.getItem("desktop_bubble_chat_messenger_accounts")).toBeNull();
  });
});
