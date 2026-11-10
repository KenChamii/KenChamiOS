import { describe, expect, it } from "vitest";
import { formatSize, parseKeep } from "./format";

describe("parseKeep", () => {
  it("takes whole numbers from 1 to 365", () => {
    expect(parseKeep(" 14 ")).toBe(14);
    expect(parseKeep("0")).toBeNull();
    expect(parseKeep("366")).toBeNull();
    expect(parseKeep("2.5")).toBeNull();
  });
});

describe("formatSize", () => {
  it("picks a readable unit", () => {
    expect(formatSize(512)).toBe("512 B");
    expect(formatSize(2048)).toBe("2 KB");
    expect(formatSize(3 * 1024 * 1024)).toBe("3.0 MB");
  });
});
