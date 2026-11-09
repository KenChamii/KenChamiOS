import { describe, expect, it } from "vitest";
import { errorMessage } from "./errors";

describe("errorMessage", () => {
  it("reads the message of an app error", () => {
    expect(errorMessage({ code: "invalid", message: "project name is required" })).toBe(
      "project name is required",
    );
  });

  it("accepts plain strings", () => {
    expect(errorMessage("boom")).toBe("boom");
  });

  it("falls back for unknown values", () => {
    expect(errorMessage(42)).toBe("Something went wrong");
  });
});
