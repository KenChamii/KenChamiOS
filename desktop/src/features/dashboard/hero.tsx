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
    <section className="flex flex-wrap items-end justify-between gap-x-16 gap-y-8 py-6">
      <Stat hero label="Due today" value={due.data?.length ?? "–"} />
      <div className="flex gap-12 pb-3">
        <Stat
          label="Focus this week"
          value={week.data ? formatDuration(week.data.focus_seconds) : "–"}
        />
        <Stat label="Active projects" value={projects.data ? active.length : "–"} />
      </div>
    </section>
  );
}
