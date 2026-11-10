import { fromDateInput, toDateInput } from "./dates";
import { addDays, addMonths, weekdayOf, type Day } from "./days";

export const REPEAT_UNITS = ["daily", "weekly", "monthly", "weekdays"] as const;
export type RepeatUnit = (typeof REPEAT_UNITS)[number];
export type RepeatRule = { unit: RepeatUnit; interval: number };

const RULE_PATTERN = /^(daily|weekly|monthly|weekdays):([1-9]\d{0,2})$/;

/** `unit:interval` as stored in `tasks.recurrence`, or null when it is not a valid rule. */
export function parseRule(text: string): RepeatRule | null {
  const match = RULE_PATTERN.exec(text);
  if (!match) return null;
  const rule = { unit: match[1] as RepeatUnit, interval: Number(match[2]) };
  return rule.unit === "weekdays" && rule.interval !== 1 ? null : rule;
}

export function formatRule(rule: RepeatRule): string {
  return `${rule.unit}:${rule.interval}`;
}

const UNIT_WORDS: Record<Exclude<RepeatUnit, "weekdays">, string> = {
  daily: "day",
  weekly: "week",
  monthly: "month",
};

export function ruleLabel(text: string): string {
  const rule = parseRule(text);
  if (!rule) return "Repeats";
  if (rule.unit === "weekdays") return "Every weekday";
  const word = UNIT_WORDS[rule.unit];
  return rule.interval === 1 ? `Every ${word}` : `Every ${rule.interval} ${word}s`;
}

function isWeekend(day: Day): boolean {
  const weekday = weekdayOf(day);
  return weekday === 0 || weekday === 6;
}

// the k-th step from the base; months count from the base so a 31st stays a 31st where it can
function step(rule: RepeatRule, base: Day, k: number): Day {
  switch (rule.unit) {
    case "daily":
      return addDays(base, k * rule.interval);
    case "weekly":
      return addDays(base, 7 * k * rule.interval);
    case "monthly":
      return addMonths(base, k * rule.interval);
    case "weekdays":
      return addDays(base, k);
  }
}

/**
 * The next due day after completing: steps from the due day (today when there is none)
 * until the result is after today. Same rules as `shared/fixtures/recurrence/rules.json`.
 */
export function nextDue(ruleText: string, due: Day | null, today: Day): Day | null {
  const rule = parseRule(ruleText);
  if (!rule) return null;
  const base = due ?? today;
  for (let k = 1; ; k += 1) {
    const candidate = step(rule, base, k);
    if (candidate > today && !(rule.unit === "weekdays" && isWeekend(candidate))) {
      return candidate;
    }
  }
}

type Repeating = { recurrence: string | null; due_at: number | null };

/** Local midnight of the next occurrence when completing the task, or null for a one-off task. */
export function nextDueAt(task: Repeating, now: number): number | null {
  if (!task.recurrence) return null;
  const due = task.due_at === null ? null : toDateInput(task.due_at);
  const next = nextDue(task.recurrence, due, toDateInput(now));
  return next === null ? null : fromDateInput(next);
}
