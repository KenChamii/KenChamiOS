import { formatDuration } from "./format";
import type { DayTotal } from "./time-stats";

type DayChartProps = { days: DayTotal[]; labeled?: boolean };

export function DayChart({ days, labeled = false }: DayChartProps) {
  const peak = Math.max(...days.map((day) => day.seconds), 1);

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex h-32 items-end gap-1" role="img" aria-label="Time worked per day">
        {days.map((day) => (
          <div
            key={day.date}
            className="flex h-full flex-1 flex-col justify-end"
            title={`${day.date}: ${formatDuration(day.seconds)}`}
          >
            <div
              className="min-h-px rounded-t bg-focus"
              style={{ height: `${(day.seconds / peak) * 100}%` }}
            />
          </div>
        ))}
      </div>
      {labeled && (
        <div className="flex gap-1" aria-hidden>
          {days.map((day) => (
            <span key={day.date} className="flex-1 text-center text-label text-muted-foreground">
              {day.date}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
