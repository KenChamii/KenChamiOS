import { describe, expect, it } from "vitest";
import { monthStart, monthWeeks } from "./calendar";

describe("month calendar", () => {
  it("starts on the monday before the first and ends on the sunday after the last", () => {
    const weeks = monthWeeks("2026-11-10");
    expect(monthStart("2026-11-10")).toBe("2026-11-01");
    expect(weeks).toHaveLength(6);
    expect(weeks[0][0]).toBe("2026-10-26");
    expect(weeks[5][6]).toBe("2026-12-06");
  });

  it("needs only four weeks for a february that starts on a monday", () => {
    const weeks = monthWeeks("2027-02-14");
    expect(weeks).toHaveLength(4);
    expect(weeks[0][0]).toBe("2027-02-01");
    expect(weeks[3][6]).toBe("2027-02-28");
  });
});
