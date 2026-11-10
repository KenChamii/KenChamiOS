import { useState } from "react";
import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { formatMinutes } from "@/features/planner/capacity";
import { projectPath } from "@/features/projects/tabs";
import { PRIORITY_LABELS } from "@/features/tasks/meta";
import { useDueTasks } from "@/hooks/use-dashboard";
import { usePlanned } from "@/hooks/use-planner";
import { useMoveTask } from "@/hooks/use-tasks";
import type { PlannedTask } from "@/lib/api/planner";
import { cn } from "@/lib/cn";
import { toDateInput } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { INBOX_ID } from "@/lib/inbox";
import { stagger } from "@/lib/motion";
import { endOfDay, startOfDay } from "./ranges";

type TodayTask = Pick<
  PlannedTask,
  "id" | "title" | "project_id" | "project_name" | "priority" | "status" | "recurrence" | "due_at"
>;

/** Today's plan first, then the other tasks due today or overdue; hidden when both are empty. */
export function TodayCard({ now }: { now: number }) {
  const today = toDateInput(now);
  const planned = usePlanned(today, today);
  const due = useDueTasks(endOfDay(now));
  const plannedIds = new Set(planned.data?.map((task) => task.id));
  const dueOnly = due.data?.filter((task) => !plannedIds.has(task.id)) ?? [];
  const error = planned.error ?? due.error;
  if (!error && !planned.data?.length && dueOnly.length === 0) return null;
  const dayStart = startOfDay(now);

  return (
    <Card className="flex flex-col gap-3">
      <div className="flex items-baseline justify-between gap-2">
        <h2 className="font-semibold tracking-tight">Today</h2>
        <Link
          to={`/notes?day=${today}`}
          className="text-sm text-muted-foreground hover:text-foreground"
        >
          Today's note
        </Link>
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <ul className="flex flex-col">
        {planned.data?.map((task, index) => (
          <TodayRow key={task.id} task={task} index={index} note={planLabel(task)} />
        ))}
        {dueOnly.map((task, index) => (
          <TodayRow
            key={task.id}
            task={task}
            index={(planned.data?.length ?? 0) + index}
            overdue={task.due_at < dayStart}
          />
        ))}
      </ul>
    </Card>
  );
}

function planLabel(task: PlannedTask): string | null {
  if (task.scheduled_at !== null) {
    return new Date(task.scheduled_at).toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  return task.duration_min === null ? null : formatMinutes(task.duration_min);
}

type TodayRowProps = { task: TodayTask; index: number; overdue?: boolean; note?: string | null };

function TodayRow({ task, index, overdue = false, note = null }: TodayRowProps) {
  const move = useMoveTask(task.project_id);
  // ticked right away; the row updates once the lists refresh
  const [done, setDone] = useState(task.status === "done");

  function complete() {
    if (done) return;
    setDone(true);
    move.mutate({ task, status: "done", beforeId: null }, { onError: () => setDone(false) });
  }

  return (
    <li className="stagger-in flex items-center gap-3 rounded-xl px-1 py-2" style={stagger(index)}>
      <Checkbox
        aria-label={`Complete ${task.title}`}
        checked={done}
        disabled={task.status === "done"}
        onCheckedChange={complete}
      />
      <Link
        to={task.project_id === INBOX_ID ? "/inbox" : projectPath(task.project_id, "tasks")}
        className="flex min-w-0 flex-1 items-center justify-between gap-2 text-sm"
      >
        <span
          className={cn(
            "truncate transition-colors duration-(--duration-base)",
            done && "text-muted-foreground line-through",
          )}
        >
          {task.title}
          <span className="text-muted-foreground"> · {task.project_name}</span>
        </span>
        <span className="flex shrink-0 items-center gap-1">
          {note && <Badge>{note}</Badge>}
          {task.priority === "high" && (
            <Badge className="text-destructive">{PRIORITY_LABELS[task.priority]}</Badge>
          )}
          {overdue && <Badge className="bg-destructive text-primary-foreground">Overdue</Badge>}
        </span>
      </Link>
    </li>
  );
}
