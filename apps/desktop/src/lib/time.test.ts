import { describe, expect, it } from "vitest";

import { ago, countdown, short } from "@/lib/time";

describe("ago", () => {
  const now = Date.parse("2026-06-30T12:00:00Z");
  const day = 86_400_000;

  it("reads the timestamps git, gh and the host send", () => {
    expect(ago("2026-06-30T13:00:00+02:00", now)).toBe("1h ago");
    expect(ago("2026-06-30T11:30:00Z", now)).toBe("30m ago");
    expect(ago(now - 2 * day, now)).toBe("2d ago");
  });

  it("calls a time ahead of this clock just now", () => {
    expect(ago("2026-06-30T12:05:00Z", now)).toBe("just now");
  });

  it("rounds the last days of a year to a year", () => {
    expect(ago(now - 300 * day, now)).toBe("10mo ago");
    expect(ago(now - 362 * day, now)).toBe("1y ago");
    expect(ago(now - 800 * day, now)).toBe("2y ago");
  });
});

describe("short", () => {
  it("shows the largest whole unit", () => {
    expect(short(30_000)).toBe("now");
    expect(short(5 * 60_000)).toBe("5m");
    expect(short(3 * 3_600_000)).toBe("3h");
    expect(short(9 * 86_400_000)).toBe("1w");
  });
});

describe("countdown", () => {
  it("shows two units until a reset", () => {
    expect(countdown(90 * 60_000)).toBe("1h 30m");
    expect(countdown(26 * 3_600_000)).toBe("1d 2h");
    expect(countdown(1_000)).toBe("1m");
  });
});
