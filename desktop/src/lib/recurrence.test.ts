import { describe, expect, it } from "vitest";
import rules from "../../../shared/fixtures/recurrence/rules.json";
import { formatRule, nextDue, nextDueAt, parseRule, ruleLabel } from "./recurrence";

describe("parseRule", () => {
  it("accepts the shared valid rules and round-trips them", () => {
    for (const text of rules.valid) {
      const rule = parseRule(text);
      expect(rule, text).not.toBeNull();
      if (rule) expect(formatRule(rule)).toBe(text);
    }
  });

  it("rejects the shared invalid rules", () => {
    for (const text of rules.invalid) expect(parseRule(text), JSON.stringify(text)).toBeNull();
  });
});

describe("nextDue", () => {
  it("matches every shared case", () => {
    for (const { rule, due, today, expected } of rules.next) {
      expect(nextDue(rule, due, today), `${rule} from ${due} on ${today}`).toBe(expected);
    }
  });

  it("is null for an unknown rule", () => {
    expect(nextDue("yearly:1", "2026-11-10", "2026-11-10")).toBeNull();
  });
});

describe("nextDueAt", () => {
  it("returns local midnight of the next day, or null for a one-off task", () => {
    const now = new Date(2026, 0, 31, 15, 30).getTime();
    const due = new Date(2026, 0, 31).getTime();
    expect(nextDueAt({ recurrence: "monthly:1", due_at: due }, now)).toBe(
      new Date(2026, 1, 28).getTime(),
    );
    expect(nextDueAt({ recurrence: null, due_at: due }, now)).toBeNull();
  });
});

describe("ruleLabel", () => {
  it("reads like a sentence", () => {
    expect(ruleLabel("daily:1")).toBe("Every day");
    expect(ruleLabel("weekly:2")).toBe("Every 2 weeks");
    expect(ruleLabel("weekdays:1")).toBe("Every weekday");
  });
});
