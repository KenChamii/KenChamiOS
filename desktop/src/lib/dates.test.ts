import { describe, expect, it } from "vitest";
import { fromDateInput, timeAgo, toDateInput } from "./dates";

describe("date input", () => {
  it("round trips a date", () => {
    expect(toDateInput(fromDateInput("2026-03-07"))).toBe("2026-03-07");
  });

  it("maps empty values to null and back", () => {
    expect(fromDateInput("")).toBeNull();
    expect(toDateInput(null)).toBe("");
  });
});

describe("timeAgo", () => {
  const now = 10 * 24 * 60 * 60 * 1000;

  it("uses the largest fitting unit", () => {
    expect(timeAgo(now - 5_000, now)).toBe("just now");
    expect(timeAgo(now - 5 * 60_000, now)).toBe("5m ago");
    expect(timeAgo(now - 3 * 3_600_000, now)).toBe("3h ago");
    expect(timeAgo(now - 2 * 86_400_000, now)).toBe("2d ago");
  });

  it("never goes negative for future times", () => {
    expect(timeAgo(now + 1_000, now)).toBe("just now");
  });
});
