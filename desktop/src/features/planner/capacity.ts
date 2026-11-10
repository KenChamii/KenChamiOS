import { formatDuration } from "@/features/focus/format";

/** Estimates offered when planning, in minutes. */
export const ESTIMATES = [15, 30, 45, 60, 90, 120, 180, 240] as const;
export const DEFAULT_ESTIMATE_MIN = 30;
export const MIN_AVAILABLE_MIN = 30;
export const MAX_AVAILABLE_MIN = 24 * 60;

type Estimated = { duration_min: number | null };

export type Capacity = {
  plannedMin: number;
  availableMin: number;
  /** minutes planned beyond what is available, 0 when it fits */
  overMin: number;
  /** tasks without an estimate, they count as 0 */
  unestimated: number;
  /** planned share of the available time, capped at 1 for the bar */
  fill: number;
};

export function capacity(tasks: Estimated[], availableMin: number): Capacity {
  const plannedMin = tasks.reduce((sum, task) => sum + (task.duration_min ?? 0), 0);
  return {
    plannedMin,
    availableMin,
    overMin: Math.max(0, plannedMin - availableMin),
    unestimated: tasks.filter((task) => task.duration_min === null).length,
    fill: availableMin > 0 ? Math.min(1, plannedMin / availableMin) : 1,
  };
}

/** Minutes as "1h 30m" or "45m". */
export function formatMinutes(minutes: number): string {
  return formatDuration(minutes * 60);
}

/** Hours typed in settings ("6", "7.5") as minutes, or null when out of range. */
export function parseHours(text: string): number | null {
  if (!/^\d+(\.\d+)?$/.test(text.trim())) return null;
  const minutes = Math.round(Number(text) * 60);
  return minutes >= MIN_AVAILABLE_MIN && minutes <= MAX_AVAILABLE_MIN ? minutes : null;
}
