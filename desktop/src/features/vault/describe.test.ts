import { describe, expect, it } from "vitest";
import { describeChange } from "./describe";

describe("describeChange", () => {
  it("names the direction and the kind", () => {
    expect(describeChange({ path: "a", kind: "added", direction: "to_vault" })).toBe(
      "New to vault",
    );
    expect(describeChange({ path: "a", kind: "updated", direction: "to_app" })).toBe(
      "Updated to app",
    );
  });

  it("says the vault wins in a conflict", () => {
    expect(describeChange({ path: "a", kind: "conflict", direction: "to_app" })).toBe(
      "Conflict, vault wins",
    );
  });
});
