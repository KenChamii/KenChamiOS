import { addDays, addMonths, mondayOf, type Day } from "@/lib/days";

/** The first day of the month holding `day`. */
export function monthStart(day: Day): Day {
  return `${day.slice(0, 8)}01`;
}

/** Whole weeks, Monday first, that cover the month holding `day`. */
export function monthWeeks(day: Day): Day[][] {
  const first = monthStart(day);
  const last = addDays(addMonths(first, 1), -1);
  const weeks: Day[][] = [];
  for (let monday = mondayOf(first); monday <= last; monday = addDays(monday, 7)) {
    weeks.push(Array.from({ length: 7 }, (_, index) => addDays(monday, index)));
  }
  return weeks;
}
