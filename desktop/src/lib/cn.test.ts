import { describe, expect, it } from "vitest";
import { cn } from "./cn";

describe("cn", () => {
  it("joins class names and skips falsy values", () => {
    expect(cn("a", false, undefined, "b")).toBe("a b");
  });

  it("lets later tailwind classes win", () => {
    expect(cn("px-2", "px-4")).toBe("px-4");
  });
});
