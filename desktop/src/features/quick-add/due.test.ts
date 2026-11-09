import { describe, expect, it } from "vitest";
import { dueAt } from "./due";

const NOW = new Date(2026, 10, 30, 21, 15).getTime();

describe("dueAt", () => {
  it("is null for no date or an empty pick", () => {
    expect(dueAt("none", "2026-12-01", NOW)).toBeNull();
    expect(dueAt("pick", "", NOW)).toBeNull();
  });

  it("is local midnight of today or tomorrow, across a month end", () => {
    expect(dueAt("today", "", NOW)).toBe(new Date(2026, 10, 30).getTime());
    expect(dueAt("tomorrow", "", NOW)).toBe(new Date(2026, 11, 1).getTime());
  });

  it("reads a picked date", () => {
    expect(dueAt("pick", "2027-01-05", NOW)).toBe(new Date(2027, 0, 5).getTime());
  });
});
