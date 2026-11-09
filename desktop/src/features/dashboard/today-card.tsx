import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { PHASE_LABELS, formatClock } from "@/features/focus/format";
import { STAGE_LABELS } from "@/features/projects/meta";
import { PRIORITY_LABELS } from "@/features/tasks/meta";
import { useDueTasks } from "@/hooks/use-dashboard";
import { useProjects } from "@/hooks/use-projects";
import { useTimer } from "@/hooks/use-timer";
import type { DueTask } from "@/lib/api/dashboard";
import { errorMessage } from "@/lib/errors";
import { endOfDay, startOfDay } from "./ranges";

type TodayCardProps = { now: number };

export function TodayCard({ now }: TodayCardProps) {
  const due = useDueTasks(endOfDay(now));
  const projects = useProjects({ search: null, tag: null, include_archived: false });
  const timer = useTimer();
  const active = (projects.data ?? []).filter((project) => project.stage !== "done");
  const today = startOfDay(now);
  const error = due.error ?? projects.error;

  return (
    <Card className="flex flex-col gap-4">
      <h2 className="font-semibold tracking-tight">Today</h2>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}

      {timer.data && (
        <Link
          to={`/projects/${timer.data.project_id}`}
          className="flex items-center gap-3 rounded-xl bg-muted px-3 py-2 text-sm"
        >
          <span className="font-medium">{PHASE_LABELS[timer.data.phase]}</span>
          <span className="font-mono tabular-nums">{formatClock(timer.data.remaining_s)}</span>
          <span className="text-muted-foreground">
            {timer.data.status === "paused" ? "Paused" : "Running"}
          </span>
        </Link>
      )}

      <section className="flex flex-col gap-1">
        <h3 className="text-sm text-muted-foreground">Tasks due</h3>
        {due.data?.length === 0 && <p className="text-sm">Nothing due or overdue.</p>}
        <ul className="flex flex-col gap-1">
          {due.data?.map((task) => (
            <DueRow key={task.id} task={task} overdue={task.due_at < today} />
          ))}
        </ul>
      </section>

      <section className="flex flex-col gap-1">
        <h3 className="text-sm text-muted-foreground">Active projects</h3>
        {active.length === 0 && <p className="text-sm">No active projects.</p>}
        <ul className="flex flex-col gap-1">
          {active.map((project) => (
            <li key={project.id}>
              <Link
                to={`/projects/${project.id}`}
                className="flex items-center justify-between gap-2 text-sm hover:underline"
              >
                <span className="truncate">{project.name}</span>
                <Badge>{STAGE_LABELS[project.stage]}</Badge>
              </Link>
            </li>
          ))}
        </ul>
      </section>
    </Card>
  );
}

function DueRow({ task, overdue }: { task: DueTask; overdue: boolean }) {
  return (
    <li>
      <Link
        to={`/projects/${task.project_id}`}
        className="flex items-center justify-between gap-2 text-sm hover:underline"
      >
        <span className="truncate">
          {task.title}
          <span className="text-muted-foreground"> · {task.project_name}</span>
        </span>
        <span className="flex shrink-0 items-center gap-1">
          {task.priority === "high" && <Badge>{PRIORITY_LABELS[task.priority]}</Badge>}
          {overdue && <Badge className="bg-destructive text-primary-foreground">Overdue</Badge>}
        </span>
      </Link>
    </li>
  );
}
