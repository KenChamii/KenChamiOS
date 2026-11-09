import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { projectPath } from "@/features/projects/tabs";
import { PRIORITY_LABELS } from "@/features/tasks/meta";
import { useDueTasks } from "@/hooks/use-dashboard";
import { useMoveTask } from "@/hooks/use-tasks";
import type { DueTask } from "@/lib/api/dashboard";
import { cn } from "@/lib/cn";
import { errorMessage } from "@/lib/errors";
import { stagger } from "@/lib/motion";
import { endOfDay, startOfDay } from "./ranges";

/** Tasks due today or overdue; hidden when there are none. */
export function TodayCard({ now }: { now: number }) {
  const due = useDueTasks(endOfDay(now));
  const today = startOfDay(now);
  if (!due.error && !due.data?.length) return null;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Today</h2>
      {due.error && <p className="text-sm text-destructive">{errorMessage(due.error)}</p>}
      <ul className="flex flex-col">
        {due.data?.map((task, index) => (
          <DueRow key={task.id} task={task} index={index} overdue={task.due_at < today} />
        ))}
      </ul>
    </Card>
  );
}

type DueRowProps = { task: DueTask; index: number; overdue: boolean };

function DueRow({ task, index, overdue }: DueRowProps) {
  const queryClient = useQueryClient();
  const move = useMoveTask(task.project_id);
  // ticked right away; the row leaves once the due list refreshes
  const [done, setDone] = useState(false);

  function complete() {
    if (done) return;
    setDone(true);
    move.mutate(
      { id: task.id, status: "done", beforeId: null },
      {
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["dashboard"] }),
        onError: () => setDone(false),
      },
    );
  }

  return (
    <li className="stagger-in flex items-center gap-3 rounded-xl px-1 py-2" style={stagger(index)}>
      <Checkbox aria-label={`Complete ${task.title}`} checked={done} onCheckedChange={complete} />
      <Link
        to={projectPath(task.project_id, "tasks")}
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
          {task.priority === "high" && (
            <Badge className="text-destructive">{PRIORITY_LABELS[task.priority]}</Badge>
          )}
          {overdue && <Badge className="bg-destructive text-primary-foreground">Overdue</Badge>}
        </span>
      </Link>
    </li>
  );
}
