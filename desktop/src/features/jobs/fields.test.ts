import { describe, expect, it } from "vitest";
import { EMPTY_JOB, canSave, fromLocalInput, toLocalInput } from "./fields";

describe("datetime-local conversion", () => {
  it("round trips to the minute", () => {
    const ms = new Date(2026, 10, 16, 9, 30).getTime();
    expect(toLocalInput(ms)).toBe("2026-11-16T09:30");
    expect(fromLocalInput("2026-11-16T09:30")).toBe(ms);
  });

  it("treats empty and broken values as no date", () => {
    expect(toLocalInput(null)).toBe("");
    expect(fromLocalInput("")).toBeNull();
    expect(fromLocalInput("nope")).toBeNull();
  });
});

describe("canSave", () => {
  it("needs a company and a role", () => {
    expect(canSave(EMPTY_JOB)).toBe(false);
    expect(canSave({ ...EMPTY_JOB, company: "Acme", role: " " })).toBe(false);
    expect(canSave({ ...EMPTY_JOB, company: "Acme", role: "Engineer" })).toBe(true);
  });
});
