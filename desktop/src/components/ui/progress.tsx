import { AREA_FILL, type Area } from "@/lib/areas";
import { cn } from "@/lib/cn";

type ProgressProps = { value: number; max: number; area?: Area };

export function Progress({ value, max, area }: ProgressProps) {
  const percent = max === 0 ? 0 : Math.round((value / max) * 100);
  return (
    <div
      role="progressbar"
      aria-valuenow={value}
      aria-valuemax={max}
      className="h-2 w-full overflow-hidden rounded-full bg-muted"
    >
      <div
        className={cn(
          "h-full transition-[width] duration-(--duration-base) ease-(--ease-out)",
          area ? AREA_FILL[area] : "bg-accent",
        )}
        style={{ width: `${percent}%` }}
      />
    </div>
  );
}
