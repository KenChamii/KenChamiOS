import { describe, expect, it } from "vitest";
import { formatTags, parseTags } from "./tags";

describe("tags", () => {
  it("splits on commas and trims", () => {
    expect(parseTags(" ui , rust,, ")).toEqual(["ui", "rust"]);
  });

  it("formats with a comma and space", () => {
    expect(formatTags(["ui", "rust"])).toBe("ui, rust");
  });
});
