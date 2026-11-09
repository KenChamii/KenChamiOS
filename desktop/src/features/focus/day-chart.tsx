import { formatDuration } from "./format";
import type { DayTotal } from "./time-stats";

export function DayChart({ days }: { days: DayTotal[] }) {
  const peak = Math.max(...days.map((day) => day.seconds), 1);

  return (
    <div className="flex h-32 items-end gap-1" role="img" aria-label="Time worked per day">
      {days.map((day) => (
        <div
          key={day.date}
          className="flex h-full flex-1 flex-col justify-end"
          title={`${day.date}: ${formatDuration(day.seconds)}`}
        >
          <div
            className="min-h-px rounded-t bg-primary"
            style={{ height: `${(day.seconds / peak) * 100}%` }}
          />
        </div>
      ))}
    </div>
  );
}
