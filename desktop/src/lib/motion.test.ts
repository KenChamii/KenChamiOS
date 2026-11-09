import { describe, expect, it } from "vitest";
import { countUpValue } from "./motion";

describe("countUpValue", () => {
  it("starts at the old value and ends on the target", () => {
    expect(countUpValue(0, 40, 0, 250)).toBe(0);
    expect(countUpValue(0, 40, 250, 250)).toBe(40);
    expect(countUpValue(0, 40, 9999, 250)).toBe(40);
  });

  it("eases out, so it is past halfway at half time", () => {
    expect(countUpValue(0, 100, 125, 250)).toBe(88);
  });

  it("counts down as well as up", () => {
    expect(countUpValue(10, 4, 250, 250)).toBe(4);
    expect(countUpValue(10, 4, 125, 250)).toBe(5);
  });

  it("jumps straight to the target with no duration", () => {
    expect(countUpValue(0, 7, 0, 0)).toBe(7);
  });
});
