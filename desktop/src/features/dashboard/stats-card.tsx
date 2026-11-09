import { Card } from "@/components/ui/card";
import { useRecap } from "@/hooks/use-dashboard";
import { errorMessage } from "@/lib/errors";
import { FocusProjects } from "./focus-projects";
import { MediaStats } from "./media-stats";
import { monthRange, weekRange } from "./ranges";

export function StatsCard({ now }: { now: number }) {
  const week = useRecap(weekRange(now));
  const month = useRecap(monthRange(now));
  const error = week.error ?? month.error;

  return (
    <Card className="flex flex-col gap-4">
      <h2 className="font-semibold tracking-tight">Stats</h2>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <section className="flex flex-col gap-2">
        <h3 className="text-sm text-muted-foreground">Focus per project this week</h3>
        <FocusProjects projects={week.data?.focus_by_project ?? []} />
      </section>
      <section className="flex flex-col gap-2">
        <h3 className="text-sm text-muted-foreground">Media this month</h3>
        <MediaStats stats={month.data?.media ?? []} />
      </section>
    </Card>
  );
}
