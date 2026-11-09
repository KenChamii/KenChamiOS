import { describe, expect, it } from "vitest";
import { idleLabel, parseIdleDays } from "./idle-days";

describe("parseIdleDays", () => {
  it.each([
    ["7", 7],
    [" 14 ", 14],
    ["1", 1],
    ["365", 365],
  ])("%s is %i", (text, expected) => {
    expect(parseIdleDays(text)).toBe(expected);
  });

  it.each(["", "0", "366", "-3", "2.5", "abc", "7 days"])("%s is refused", (text) => {
    expect(parseIdleDays(text)).toBeNull();
  });
});

describe("idleLabel", () => {
  it.each([
    [0, "Quiet since today"],
    [1, "Quiet for 1 day"],
    [9, "Quiet for 9 days"],
  ])("%i days reads %s", (days, expected) => {
    expect(idleLabel(days)).toBe(expected);
  });
});
