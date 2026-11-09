import { describe, expect, it } from "vitest";
import { endOfDay, monthRange, startOfDay, weekRange, yearRange } from "./ranges";

const local = (year: number, month: number, day: number, hour = 0, minute = 0) =>
  new Date(year, month - 1, day, hour, minute).getTime();

describe("startOfDay", () => {
  it("is the local midnight at or before the time", () => {
    expect(startOfDay(local(2026, 11, 9, 23, 59))).toBe(local(2026, 11, 9));
    expect(startOfDay(local(2026, 11, 9))).toBe(local(2026, 11, 9));
  });
});

describe("endOfDay", () => {
  it("is the next local midnight, even a minute before it", () => {
    expect(endOfDay(local(2026, 11, 9, 8))).toBe(local(2026, 11, 10));
    expect(endOfDay(local(2026, 11, 9, 23, 59))).toBe(local(2026, 11, 10));
  });

  it("moves on at midnight itself", () => {
    expect(endOfDay(local(2026, 11, 10))).toBe(local(2026, 11, 11));
  });

  it("rolls over month and year ends", () => {
    expect(endOfDay(local(2026, 12, 31, 12))).toBe(local(2027, 1, 1));
    expect(endOfDay(local(2024, 2, 28, 12))).toBe(local(2024, 2, 29));
  });
});

describe("weekRange", () => {
  it("runs Monday to Monday with a boundary per day", () => {
    // 2026-11-09 is a Monday
    const range = weekRange(local(2026, 11, 11, 15));

    expect(range.from).toBe(local(2026, 11, 9));
    expect(range.to).toBe(local(2026, 11, 16));
    expect(range.buckets).toHaveLength(8);
    expect(range.buckets[3]).toBe(local(2026, 11, 12));
  });

  it("puts Sunday evening in the week that started six days earlier", () => {
    const range = weekRange(local(2026, 11, 15, 23, 59));

    expect(range.from).toBe(local(2026, 11, 9));
  });

  it("starts a new week at Monday midnight", () => {
    expect(weekRange(local(2026, 11, 16)).from).toBe(local(2026, 11, 16));
  });

  it("crosses a month end", () => {
    const range = weekRange(local(2026, 12, 2));

    expect(range.from).toBe(local(2026, 11, 30));
    expect(range.buckets[2]).toBe(local(2026, 12, 2));
  });
});

describe("monthRange", () => {
  it("covers the calendar month", () => {
    const range = monthRange(local(2026, 2, 28, 23, 59));

    expect(range).toEqual({
      from: local(2026, 2, 1),
      to: local(2026, 3, 1),
      buckets: [local(2026, 2, 1), local(2026, 3, 1)],
    });
  });

  it("handles December", () => {
    expect(monthRange(local(2026, 12, 31)).to).toBe(local(2027, 1, 1));
  });
});

describe("yearRange", () => {
  it("has twelve monthly buckets from January to the next January", () => {
    const range = yearRange(2026);

    expect(range.from).toBe(local(2026, 1, 1));
    expect(range.to).toBe(local(2027, 1, 1));
    expect(range.buckets).toHaveLength(13);
    expect(range.buckets[1]).toBe(local(2026, 2, 1));
    expect(range.buckets[11]).toBe(local(2026, 12, 1));
  });
});
