import { describe, expect, it } from "vitest";
import { branchLabel, commitLabel, shortHash } from "./meta";

describe("shortHash", () => {
  it("keeps the first 7 characters", () => {
    expect(shortHash("0123456789abcdef")).toBe("0123456");
    expect(shortHash("abc")).toBe("abc");
  });
});

describe("branchLabel", () => {
  it("prefers the branch name", () => {
    expect(branchLabel({ branch: "main", head: "abc" })).toBe("main");
  });

  it("tells detached HEAD from an empty repo", () => {
    expect(branchLabel({ branch: null, head: "abc" })).toBe("Detached HEAD");
    expect(branchLabel({ branch: null, head: null })).toBe("No commits yet");
  });
});

describe("commitLabel", () => {
  it("joins branch and short hash", () => {
    expect(commitLabel("main", "0123456789")).toBe("main @ 0123456");
  });

  it("handles detached and missing git data", () => {
    expect(commitLabel(null, "0123456789")).toBe("@ 0123456");
    expect(commitLabel("main", null)).toBe("main");
    expect(commitLabel(null, null)).toBeNull();
  });
});
