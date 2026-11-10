import { describe, expect, it } from "vitest";
import { capacity, formatMinutes, parseHours } from "./capacity";

const est = (...minutes: (number | null)[]) => minutes.map((duration_min) => ({ duration_min }));

describe("capacity", () => {
  it("sums the estimates against the available time", () => {
    expect(capacity(est(30, 90, null), 360)).toEqual({
      plannedMin: 120,
      availableMin: 360,
      overMin: 0,
      unestimated: 1,
      fill: 1 / 3,
    });
  });

  it("reports how far over the plan is and caps the bar", () => {
    const over = capacity(est(240, 180), 360);
    expect(over.overMin).toBe(60);
    expect(over.fill).toBe(1);
  });

  it("fits exactly at the limit and handles an empty plan", () => {
    expect(capacity(est(360), 360).overMin).toBe(0);
    expect(capacity([], 360)).toMatchObject({ plannedMin: 0, fill: 0, unestimated: 0 });
  });

  it("formats minutes as hours", () => {
    expect(formatMinutes(90)).toBe("1h 30m");
    expect(formatMinutes(45)).toBe("45m");
  });

  it("reads hours from settings", () => {
    expect(parseHours("6")).toBe(360);
    expect(parseHours(" 7.5 ")).toBe(450);
    expect(parseHours("0.25")).toBeNull();
    expect(parseHours("25")).toBeNull();
    expect(parseHours("six")).toBeNull();
  });
});
