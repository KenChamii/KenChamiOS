import { Card } from "@/components/ui/card";
import { Stat } from "@/components/layout/stat";
import { formatDuration } from "@/features/focus/format";
import { useDueTasks, useRecap } from "@/hooks/use-dashboard";
import { useProjects } from "@/hooks/use-projects";
import { endOfDay, weekRange } from "./ranges";

export function Hero({ now }: { now: number }) {
  const due = useDueTasks(endOfDay(now));
  const week = useRecap(weekRange(now));
  const projects = useProjects({ search: null, tag: null, include_archived: false });
  const active = (projects.data ?? []).filter((project) => project.stage !== "done");

  return (
    <Card className="flex flex-wrap items-end gap-x-16 gap-y-6 p-8">
      <Stat hero label="Due today" value={due.data?.length ?? "–"} />
      <Stat
        hero
        label="Focus this week"
        value={week.data ? formatDuration(week.data.focus_seconds) : "–"}
      />
      <Stat label="Active projects" value={projects.data ? active.length : "–"} />
    </Card>
  );
}
