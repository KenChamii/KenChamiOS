import { describe, expect, it } from "vitest";
import { formatClock, formatDuration } from "./format";

describe("formatClock", () => {
  it.each([
    [0, "0:00"],
    [9, "0:09"],
    [1449, "24:09"],
    [3600, "1:00:00"],
    [3725, "1:02:05"],
    [-5, "0:00"],
  ])("%i seconds is %s", (seconds, expected) => {
    expect(formatClock(seconds)).toBe(expected);
  });
});

describe("formatDuration", () => {
  it.each([
    [0, "0m"],
    [29, "0m"],
    [30, "1m"],
    [45 * 60, "45m"],
    [7500, "2h 05m"],
  ])("%i seconds is %s", (seconds, expected) => {
    expect(formatDuration(seconds)).toBe(expected);
  });
});
