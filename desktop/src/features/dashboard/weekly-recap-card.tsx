import { Card } from "@/components/ui/card";
import { DayChart } from "@/features/focus/day-chart";
import { formatDuration } from "@/features/focus/format";
import { useRecap } from "@/hooks/use-dashboard";
import { toDateInput } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { FocusProjects } from "./focus-projects";
import { MediaStats } from "./media-stats";
import { weekRange } from "./ranges";

export function WeeklyRecapCard({ now }: { now: number }) {
  const range = weekRange(now);
  const recap = useRecap(range);
  const view = recap.data;
  const days = (view?.focus_by_bucket ?? []).map((seconds, index) => ({
    date: toDateInput(range.buckets[index]),
    seconds,
  }));

  return (
    <Card className="flex flex-col gap-4">
      <h2 className="font-medium">This week</h2>
      {recap.error && <p className="text-sm text-destructive">{errorMessage(recap.error)}</p>}
      {view && (
        <>
          <p className="text-sm">
            {formatDuration(view.focus_seconds)} focused
            {view.stage_changes > 0 && `, ${view.stage_changes} project stage moves`}
            {view.projects_finished.length > 0 && `, finished ${view.projects_finished.join(", ")}`}
            .
          </p>
          <DayChart days={days} />
          <FocusProjects projects={view.focus_by_project} />
          <MediaStats stats={view.media} />
        </>
      )}
    </Card>
  );
}
