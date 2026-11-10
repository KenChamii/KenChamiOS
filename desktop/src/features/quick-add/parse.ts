import type { Priority } from "@/lib/api/tasks";
import { addDays, isDay, weekdayOf, type Day } from "@/lib/days";

export type QuickTask = {
  title: string;
  due: Day | null;
  priority: Priority | null;
  /** the text after #, as typed */
  project: string | null;
  recurrence: string | null;
};

const WEEKDAYS = ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];

// maps, so words like "constructor" never hit an object prototype
const PRIORITY_WORDS = new Map<string, Priority>([
  ["!high", "high"],
  ["!h", "high"],
  ["!medium", "medium"],
  ["!med", "medium"],
  ["!m", "medium"],
  ["!low", "low"],
  ["!l", "low"],
]);

const SINGLE_REPEATS = new Map(
  Object.entries({
    daily: "daily:1",
    weekly: "weekly:1",
    monthly: "monthly:1",
    weekdays: "weekdays:1",
  }),
);

const EVERY_UNITS = new Map(
  Object.entries({
    day: "daily",
    days: "daily",
    week: "weekly",
    weeks: "weekly",
    month: "monthly",
    months: "monthly",
  }),
);

// the next date with this weekday, strictly after today, or from today on when `orToday`
function nextWeekday(today: Day, weekday: number, orToday: boolean): Day {
  const ahead = (weekday - weekdayOf(today) + 7) % 7;
  return addDays(today, ahead === 0 && !orToday ? 7 : ahead);
}

function isWeekend(day: Day): boolean {
  const weekday = weekdayOf(day);
  return weekday === 0 || weekday === 6;
}

function count(word: string | undefined): number | null {
  return word && /^[1-9]\d{0,2}$/.test(word) ? Number(word) : null;
}

type Repeat = { rule: string; due: Day | null; length: number };

// "daily", "every 3 days", "every weekday", "every monday"
function readRepeat(words: string[], at: number, today: Day): Repeat | null {
  const word = words[at];
  const single = SINGLE_REPEATS.get(word);
  if (single) return { rule: single, due: null, length: 1 };
  if (word !== "every") return null;
  const next = words[at + 1];
  if (next === "weekday") return { rule: "weekdays:1", due: null, length: 2 };
  const everyUnit = EVERY_UNITS.get(next);
  if (everyUnit) return { rule: `${everyUnit}:1`, due: null, length: 2 };
  const weekday = WEEKDAYS.indexOf(next);
  if (weekday >= 0) {
    return { rule: "weekly:1", due: nextWeekday(today, weekday, true), length: 2 };
  }
  const interval = count(next);
  const unit = EVERY_UNITS.get(words[at + 2]);
  if (interval !== null && unit) {
    return { rule: `${unit}:${interval}`, due: null, length: 3 };
  }
  return null;
}

type DueMatch = { due: Day; length: number };

// "today", "tmr", "friday", "on friday", "next friday", "next week", "in 3 days", "2026-12-01"
function readDue(words: string[], original: string[], at: number, today: Day): DueMatch | null {
  const word = words[at];
  if (word === "today") return { due: today, length: 1 };
  if (word === "tomorrow" || word === "tmr") return { due: addDays(today, 1), length: 1 };
  if (isDay(original[at])) return { due: original[at], length: 1 };
  const lead = word === "on" || word === "next" ? 1 : 0;
  const weekday = WEEKDAYS.indexOf(words[at + lead]);
  if (weekday >= 0) return { due: nextWeekday(today, weekday, false), length: lead + 1 };
  if (word === "next" && words[at + 1] === "week") {
    return { due: nextWeekday(today, 1, false), length: 2 };
  }
  const amount = count(words[at + 1]);
  const unit = words[at + 2];
  if (word === "in" && amount !== null && (unit === "day" || unit === "days")) {
    return { due: addDays(today, amount), length: 3 };
  }
  if (word === "in" && amount !== null && (unit === "week" || unit === "weeks")) {
    return { due: addDays(today, 7 * amount), length: 3 };
  }
  return null;
}

/**
 * Reads dates, `!priority`, `#project` and repeats out of a quick-add line, left to right;
 * the first match of each kind wins and the rest stays in the title.
 * Cases shared with Android: `shared/fixtures/quick-add/parse.json`.
 */
export function parseQuickAdd(input: string, today: Day): QuickTask {
  const original = input.trim().split(/\s+/).filter(Boolean);
  const words = original.map((word) => word.toLowerCase());
  const title: string[] = [];
  let due: Day | null = null;
  let repeatDue: Day | null = null;
  let priority: Priority | null = null;
  let project: string | null = null;
  let recurrence: string | null = null;

  let at = 0;
  while (at < words.length) {
    const repeat: Repeat | null = recurrence === null ? readRepeat(words, at, today) : null;
    if (repeat) {
      recurrence = repeat.rule;
      repeatDue = repeat.due;
      at += repeat.length;
      continue;
    }
    const dueMatch: DueMatch | null = due === null ? readDue(words, original, at, today) : null;
    if (dueMatch) {
      due = dueMatch.due;
      at += dueMatch.length;
      continue;
    }
    const word = words[at];
    const wordPriority = PRIORITY_WORDS.get(word);
    if (priority === null && wordPriority) {
      priority = wordPriority;
    } else if (project === null && /^#[\p{L}\p{N}_-]+$/u.test(word)) {
      project = original[at].slice(1);
    } else {
      title.push(original[at]);
    }
    at += 1;
  }

  if (due === null) due = repeatDue;
  if (due === null && recurrence !== null) {
    due = recurrence === "weekdays:1" && isWeekend(today) ? nextWeekday(today, 1, false) : today;
  }
  return { title: title.join(" "), due, priority, project, recurrence };
}
