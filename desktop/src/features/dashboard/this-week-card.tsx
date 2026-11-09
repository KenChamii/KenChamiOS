import { Card } from "@/components/ui/card";
import { DayChart } from "@/features/focus/day-chart";
import { formatDuration } from "@/features/focus/format";
import { useRecap } from "@/hooks/use-dashboard";
import { errorMessage } from "@/lib/errors";
import { FocusProjects } from "./focus-projects";
import { weekRange } from "./ranges";

/** Focus per day this week; hidden until some focus time is logged. */
export function ThisWeekCard({ now }: { now: number }) {
  const range = weekRange(now);
  const recap = useRecap(range);
  const view = recap.data;
  if (recap.error) return <p className="text-sm text-destructive">{errorMessage(recap.error)}</p>;
  if (!view || view.focus_seconds === 0) return null;

  const days = view.focus_by_bucket.map((seconds, index) => ({
    date: new Date(range.buckets[index]).toLocaleDateString(undefined, { weekday: "short" }),
    seconds,
  }));

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex items-baseline justify-between gap-2">
        <h2 className="font-semibold tracking-tight">This week</h2>
        <span className="text-sm text-muted-foreground">
          {formatDuration(view.focus_seconds)} focused
        </span>
      </div>
      <DayChart days={days} labeled />
      <FocusProjects projects={view.focus_by_project} />
    </Card>
  );
}
