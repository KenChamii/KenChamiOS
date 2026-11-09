import { describe, expect, it } from "vitest";
import { addDays, addMonths, isDay, weekdayOf } from "./days";

describe("days", () => {
  it("knows real calendar days", () => {
    expect(isDay("2024-02-29")).toBe(true);
    expect(isDay("2026-02-29")).toBe(false);
    expect(isDay("2026-13-01")).toBe(false);
    expect(isDay("2026-1-01")).toBe(false);
  });

  it("adds days across month and year ends", () => {
    expect(addDays("2026-12-31", 1)).toBe("2027-01-01");
    expect(addDays("2026-03-01", -1)).toBe("2026-02-28");
  });

  it("clamps months to the month end", () => {
    expect(addMonths("2026-01-31", 1)).toBe("2026-02-28");
    expect(addMonths("2026-01-31", 2)).toBe("2026-03-31");
    expect(addMonths("2026-11-15", 3)).toBe("2027-02-15");
  });

  it("names the weekday", () => {
    expect(weekdayOf("2026-11-10")).toBe(2);
    expect(weekdayOf("2026-11-15")).toBe(0);
  });
});
