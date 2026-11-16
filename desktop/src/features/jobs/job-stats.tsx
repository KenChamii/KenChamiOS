import { Card } from "@/components/ui/card";
import { Stat } from "@/components/layout/stat";
import { useJobStats } from "@/hooks/use-jobs";
import { STAGE_LABELS } from "./stages";

function weekLabel(monday: string): string {
  const [year, month, day] = monday.split("-").map(Number);
  return new Date(year, month - 1, day).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
  });
}

export function JobStats() {
  const stats = useJobStats();
  if (!stats.data) return null;
  const { per_week, applied, responded, response_rate, time_per_stage } = stats.data;
  const busiest = Math.max(1, ...per_week.map(([, count]) => count));

  return (
    <Card className="flex flex-col gap-4">
      <h2 className="font-semibold tracking-tight">Search so far</h2>
      <div className="grid grid-cols-3 gap-3">
        <Stat label="Applied" value={String(applied)} />
        <Stat label="Heard back" value={String(responded)} />
        <Stat
          label="Response rate"
          value={response_rate === null ? "-" : `${Math.round(response_rate * 100)}%`}
        />
      </div>
      <ul className="grid grid-cols-12 items-end gap-1.5" style={{ height: "5rem" }}>
        {per_week.map(([monday, count]) => (
          <li
            key={monday}
            title={`Week of ${weekLabel(monday)}: ${count}`}
            className="flex h-full flex-col items-center justify-end gap-1"
          >
            <span className="text-xs tabular-nums text-muted-foreground">{count || ""}</span>
            <span
              className="w-full rounded-md bg-projects"
              style={{ height: `${(count / busiest) * 100}%`, minHeight: count ? 4 : 0 }}
            />
          </li>
        ))}
      </ul>
      {time_per_stage.length > 0 && (
        <ul className="flex flex-wrap gap-x-4 gap-y-1 text-sm text-muted-foreground">
          {time_per_stage.map(({ stage, avg_days }) => (
            <li key={stage}>
              {STAGE_LABELS[stage]}: {avg_days < 1 ? "under a day" : `${avg_days.toFixed(1)} days`}
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
