import type { Phase } from "@/lib/api/timer";

export const PHASE_LABELS: Record<Phase, string> = {
  focus: "Focus",
  break: "Break",
  long_break: "Long break",
};

/** Seconds as m:ss, or h:mm:ss from an hour up. */
export function formatClock(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = String(seconds % 60).padStart(2, "0");
  if (hours > 0) return `${hours}:${String(minutes).padStart(2, "0")}:${rest}`;
  return `${minutes}:${rest}`;
}

/** Seconds as "2h 05m", "45m" or "0m". */
export function formatDuration(totalSeconds: number): string {
  const minutes = Math.round(Math.max(0, totalSeconds) / 60);
  const hours = Math.floor(minutes / 60);
  if (hours === 0) return `${minutes}m`;
  return `${hours}h ${String(minutes % 60).padStart(2, "0")}m`;
}
