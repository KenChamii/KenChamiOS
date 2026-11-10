import { cn } from "@/lib/cn";
import { formatMinutes, type Capacity } from "./capacity";

/** Planned time against the hours available today, with a warning when it does not fit. */
export function CapacityBar({ capacity }: { capacity: Capacity }) {
  const over = capacity.overMin > 0;
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-baseline justify-between gap-2 text-sm">
        <span className="font-medium tabular-nums">
          {formatMinutes(capacity.plannedMin)} of {formatMinutes(capacity.availableMin)}
        </span>
        {capacity.unestimated > 0 && (
          <span className="text-muted-foreground">{capacity.unestimated} without an estimate</span>
        )}
      </div>
      <div
        role="progressbar"
        aria-label="Planned time"
        aria-valuenow={capacity.plannedMin}
        aria-valuemax={capacity.availableMin}
        className="h-2 w-full overflow-hidden rounded-full bg-muted"
      >
        <div
          className={cn(
            "h-full transition-[width] duration-(--duration-base) ease-(--ease-out)",
            over ? "bg-destructive" : "bg-focus",
          )}
          style={{ width: `${capacity.fill * 100}%` }}
        />
      </div>
      {over && (
        <p className="text-sm text-destructive">
          {formatMinutes(capacity.overMin)} more than you have. Move something to another day.
        </p>
      )}
    </div>
  );
}
