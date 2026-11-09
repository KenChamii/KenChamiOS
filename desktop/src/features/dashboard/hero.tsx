import { FolderKanban, Timer } from "lucide-react";
import { Stat } from "@/components/layout/stat";
import { IconChip } from "@/components/ui/icon-chip";
import { formatDuration } from "@/features/focus/format";
import { useCountUp } from "@/hooks/use-count-up";
import { useDueTasks, useRecap } from "@/hooks/use-dashboard";
import { useProjects } from "@/hooks/use-projects";
import { endOfDay, weekRange } from "./ranges";

export function Hero({ now }: { now: number }) {
  const due = useDueTasks(endOfDay(now));
  const week = useRecap(weekRange(now));
  const projects = useProjects({ search: null, tag: null, include_archived: false });
  const active = (projects.data ?? []).filter((project) => project.stage !== "done");
  const dueShown = useCountUp(due.data?.length ?? 0);
  const focusShown = useCountUp(week.data?.focus_seconds ?? 0);
  const activeShown = useCountUp(active.length);

  return (
    <section className="flex flex-wrap items-end justify-between gap-x-16 gap-y-8">
      <Stat hero label="Due today" value={due.data ? dueShown : "–"} />
      <div className="flex gap-12 pb-4">
        <div className="flex items-end gap-3">
          <IconChip icon={Timer} area="focus" />
          <Stat label="Focus this week" value={week.data ? formatDuration(focusShown) : "–"} />
        </div>
        <div className="flex items-end gap-3">
          <IconChip icon={FolderKanban} area="projects" />
          <Stat label="Active projects" value={projects.data ? activeShown : "–"} />
        </div>
      </div>
    </section>
  );
}
