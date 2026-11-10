/** A local calendar day as `YYYY-MM-DD`, the form habits, repeats and quick add share with Android. */
export type Day = string;

const DAY_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/;

// calendar math runs in UTC so daylight saving never shifts a date
function toUtc(day: Day): Date | null {
  const match = DAY_PATTERN.exec(day);
  if (!match) return null;
  const [year, month, date] = [Number(match[1]), Number(match[2]), Number(match[3])];
  const value = new Date(Date.UTC(year, month - 1, date));
  const real =
    value.getUTCFullYear() === year &&
    value.getUTCMonth() === month - 1 &&
    value.getUTCDate() === date;
  return real ? value : null;
}

function fromUtc(value: Date): Day {
  const month = String(value.getUTCMonth() + 1).padStart(2, "0");
  const date = String(value.getUTCDate()).padStart(2, "0");
  return `${String(value.getUTCFullYear()).padStart(4, "0")}-${month}-${date}`;
}

export function isDay(text: string): boolean {
  return toUtc(text) !== null;
}

function utcOrThrow(day: Day): Date {
  const value = toUtc(day);
  if (!value) throw new Error(`not a calendar day: ${day}`);
  return value;
}

export function addDays(day: Day, days: number): Day {
  const value = utcOrThrow(day);
  value.setUTCDate(value.getUTCDate() + days);
  return fromUtc(value);
}

/** Same day of the month, or the month's last day when it is shorter (Jan 31 + 1 is Feb 28). */
export function addMonths(day: Day, months: number): Day {
  const value = utcOrThrow(day);
  const target = new Date(Date.UTC(value.getUTCFullYear(), value.getUTCMonth() + months, 1));
  const lastDay = new Date(
    Date.UTC(target.getUTCFullYear(), target.getUTCMonth() + 1, 0),
  ).getUTCDate();
  target.setUTCDate(Math.min(value.getUTCDate(), lastDay));
  return fromUtc(target);
}

/** 0 is Sunday, 6 is Saturday. */
export function weekdayOf(day: Day): number {
  return utcOrThrow(day).getUTCDay();
}

/** The Monday on or before `day`. */
export function mondayOf(day: Day): Day {
  return addDays(day, -((weekdayOf(day) + 6) % 7));
}
