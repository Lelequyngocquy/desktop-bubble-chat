import { describe, expect, it } from "vitest";
import { isValidThreadId, normalizeThreadId } from "../threadValidation";
import { cn } from "../cn";

describe("threadValidation", () => {
  it("accepts valid alphanumeric and symbolic thread IDs and trims whitespace", () => {
    expect(normalizeThreadId("  1000123456789  ")).toBe("1000123456789");
    expect(normalizeThreadId("thread_abc.01-xyz")).toBe("thread_abc.01-xyz");
    expect(isValidThreadId("987654321")).toBe(true);
  });

  it("rejects empty values, synthetic prefixes, and unsafe characters", () => {
    expect(normalizeThreadId(null)).toBeUndefined();
    expect(normalizeThreadId(undefined)).toBeUndefined();
    expect(normalizeThreadId("   ")).toBeUndefined();
    expect(normalizeThreadId("sender-1-alice")).toBeUndefined();
    expect(normalizeThreadId("account-msg-01")).toBeUndefined();
    expect(normalizeThreadId("123/../../etc")).toBeUndefined();
    expect(normalizeThreadId("abc?param=1")).toBeUndefined();
    expect(isValidThreadId("sender-1-bob")).toBe(false);
  });
});

describe("cn utility", () => {
  it("merges conflicting Tailwind utility classes cleanly", () => {
    expect(cn("px-2 text-sm", undefined, false, "px-4")).toBe("text-sm px-4");
  });

  it("preserves custom font-size tokens alongside theme text colors", () => {
    expect(cn("inline-flex text-caption rounded-pill", "bg-status-successBg text-status-success")).toBe(
      "inline-flex text-caption rounded-pill bg-status-successBg text-status-success",
    );
    expect(cn("rounded-panel text-caption", "text-content-secondary", "rounded-none")).toBe(
      "text-caption text-content-secondary rounded-none",
    );
    expect(cn("text-nav leading-tight", "font-semibold text-primary-foreground")).toBe(
      "text-nav leading-tight font-semibold text-primary-foreground",
    );
    expect(cn("text-small font-bold", "text-content-muted")).toBe(
      "text-small font-bold text-content-muted",
    );
    expect(cn("text-body leading-tight", "font-semibold text-primary")).toBe(
      "text-body leading-tight font-semibold text-primary",
    );
  });
});
