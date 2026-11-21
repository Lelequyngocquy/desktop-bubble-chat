import { beforeEach, describe, expect, it, vi } from "vitest";

const { mockInvoke, memoryStorage } = vi.hoisted(() => {
  const map = new Map<string, string>();
  const storage = {
    getItem: (key: string) => map.get(key) ?? null,
    setItem: (key: string, value: string) => {
      map.set(key, value);
    },
    removeItem: (key: string) => {
      map.delete(key);
    },
    clear: () => map.clear(),
  };
  Object.defineProperty(globalThis, "localStorage", {
    value: storage,
    configurable: true,
    writable: true,
  });
  Object.defineProperty(globalThis, "window", {
    value: {
      localStorage: storage,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      matchMedia: vi.fn(() => ({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() })),
    },
    configurable: true,
    writable: true,
  });
  return { mockInvoke: vi.fn(), memoryStorage: storage };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import i18n from "../../i18n";
import { useSettingsStore } from "../../stores/settingsStore";
import { applyBackendUserSettings } from "../userSettingsSyncService";

describe("userSettingsSyncService", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
    memoryStorage.clear();
    useSettingsStore.setState({
      language: "en",
      themeMode: "system",
      bubbleSize: "md",
      bubblePositionMode: "auto_tuck",
      updateMode: "auto",
      isAlwaysOnTop: true,
      isAutoTrimEnabled: true,
      isGlobalShortcutEnabled: true,
      isNotificationsEnabled: true,
      closeWindowAction: "ask",
    });
    i18n.changeLanguage("en");
  });

  it("applies backend SSOT snapshot to store and i18n without echoing IPC", () => {
    applyBackendUserSettings({
      locale: "vi",
      themeMode: "dark",
      bubbleSize: "lg",
      bubblePositionMode: "free",
      updateMode: "manual",
      isAlwaysOnTop: false,
      isAutoTrimEnabled: false,
      isGlobalShortcutEnabled: false,
    });
    const state = useSettingsStore.getState();
    expect(state.language).toBe("vi");
    expect(i18n.language).toBe("vi");
    expect(state.themeMode).toBe("dark");
    expect(state.bubbleSize).toBe("lg");
    expect(state.bubblePositionMode).toBe("free");
    expect(state.updateMode).toBe("manual");
    expect(state.isAlwaysOnTop).toBe(false);
    expect(state.isAutoTrimEnabled).toBe(false);
    expect(state.isGlobalShortcutEnabled).toBe(false);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("excludes backend-owned SSOT fields from localStorage partialize", () => {
    useSettingsStore.getState().setCloseWindowAction("hide_to_tray");
    useSettingsStore.getState().setLanguage("vi", true);
    const raw = memoryStorage.getItem("desktop_bubble_chat_settings");
    expect(raw).not.toBeNull();
    const persisted = JSON.parse(raw!).state as Record<string, unknown>;
    expect(Object.keys(persisted).sort()).toEqual(["chatWindowSettings", "closeWindowAction"]);
    expect(persisted.closeWindowAction).toBe("hide_to_tray");
    expect("language" in persisted).toBe(false);
    expect("themeMode" in persisted).toBe(false);
    expect("updateMode" in persisted).toBe(false);
  });
});
