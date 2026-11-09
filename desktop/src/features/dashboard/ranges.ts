import { startOfWeek } from "@/features/focus/time-stats";
import type { RecapRange } from "@/lib/api/dashboard";

// every boundary is built from local calendar parts, so days and months follow the local time zone

/** Local midnight that starts the day containing `ms`. */
export function startOfDay(ms: number): number {
  const date = new Date(ms);
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

/** Local midnight that ends the day containing `ms`. */
export function endOfDay(ms: number): number {
  const date = new Date(ms);
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1).getTime();
}

/** Monday to Monday around `ms`, one bucket per day. */
export function weekRange(ms: number): RecapRange {
  const start = new Date(startOfWeek(ms));
  const buckets = Array.from({ length: 8 }, (_, day) =>
    new Date(start.getFullYear(), start.getMonth(), start.getDate() + day).getTime(),
  );
  return { from: buckets[0], to: buckets[7], buckets };
}

/** The calendar month containing `ms`, as one bucket. */
export function monthRange(ms: number): RecapRange {
  const date = new Date(ms);
  const from = new Date(date.getFullYear(), date.getMonth(), 1).getTime();
  const to = new Date(date.getFullYear(), date.getMonth() + 1, 1).getTime();
  return { from, to, buckets: [from, to] };
}

/** January to December of `year`, one bucket per month. */
export function yearRange(year: number): RecapRange {
  const buckets = Array.from({ length: 13 }, (_, month) => new Date(year, month, 1).getTime());
  return { from: buckets[0], to: buckets[12], buckets };
}
