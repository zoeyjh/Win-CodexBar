import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { formatWindowCountdown } from "./detailShared";

describe("formatWindowCountdown", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-01T10:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns a relative countdown when relative is true", () => {
    expect(formatWindowCountdown("2026-06-01T11:30:00Z", true)).toBe("1h 30m");
  });

  it("returns placeholders for missing or elapsed windows in relative mode", () => {
    expect(formatWindowCountdown(null, true)).toBe("--");
    expect(formatWindowCountdown("2026-06-01T09:00:00Z", true)).toBe("now");
  });

  it("returns an absolute time without the relative countdown when relative is false", () => {
    const text = formatWindowCountdown("2026-06-01T11:30:00Z", false);
    expect(text).not.toBe("1h 30m");
    expect(text).not.toMatch(/^\d+h \d+m$/);
    expect(text).not.toBe("--");
  });

  it("still returns the placeholder for a missing window in absolute mode", () => {
    expect(formatWindowCountdown(null, false)).toBe("--");
  });
});
