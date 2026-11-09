import { describe, expect, it } from "vitest";
import { progressText } from "./meta";

describe("progressText", () => {
  it("shows the total when it is known", () => {
    expect(progressText({ media_type: "anime", progress: 12, progress_total: 28 })).toBe("12 / 28");
  });

  it("shows only the count when the total is unknown", () => {
    expect(progressText({ media_type: "manga", progress: 140, progress_total: null })).toBe("140");
  });

  it("counts games in hours", () => {
    expect(progressText({ media_type: "game", progress: 30, progress_total: null })).toBe("30 h");
  });
});
