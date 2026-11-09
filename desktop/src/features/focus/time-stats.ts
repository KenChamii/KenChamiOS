import type { TimeEntry } from "@/lib/api/time-entries";
import { toDateInput } from "@/lib/dates";

export const CHART_DAYS = 14;

export type DayTotal = { date: string; seconds: number };
export type TaskTotal = { taskId: string | null; seconds: number };

export type TimeSummary = {
  totalSeconds: number;
  weekSeconds: number;
  perTask: TaskTotal[];
  days: DayTotal[];
};

/** Local midnight of the Monday on or before `ms`. */
export function startOfWeek(ms: number): number {
  const date = new Date(ms);
  const sinceMonday = (date.getDay() + 6) % 7;
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() - sinceMonday).getTime();
}

function lastDays(now: number, count: number): string[] {
  const today = new Date(now);
  return Array.from({ length: count }, (_, index) =>
    toDateInput(
      new Date(
        today.getFullYear(),
        today.getMonth(),
        today.getDate() - (count - 1 - index),
      ).getTime(),
    ),
  );
}

/** Breaks never count as worked time. */
export function summarize(entries: TimeEntry[], now: number): TimeSummary {
  const weekStart = startOfWeek(now);
  const perTask = new Map<string | null, number>();
  const perDay = new Map<string, number>();
  let totalSeconds = 0;
  let weekSeconds = 0;

  for (const entry of entries) {
    if (entry.kind === "break") continue;
    totalSeconds += entry.duration_s;
    if (entry.started_at >= weekStart) weekSeconds += entry.duration_s;
    perTask.set(entry.task_id, (perTask.get(entry.task_id) ?? 0) + entry.duration_s);
    const day = toDateInput(entry.started_at);
    perDay.set(day, (perDay.get(day) ?? 0) + entry.duration_s);
  }

  return {
    totalSeconds,
    weekSeconds,
    perTask: [...perTask]
      .map(([taskId, seconds]) => ({ taskId, seconds }))
      .sort((a, b) => b.seconds - a.seconds),
    days: lastDays(now, CHART_DAYS).map((date) => ({ date, seconds: perDay.get(date) ?? 0 })),
  };
}
