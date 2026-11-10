import { Plus, X } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useAvailableMinutes, useCandidates, usePlanned, useSetPlan } from "@/hooks/use-planner";
import type { PlannedTask } from "@/lib/api/planner";
import { toDateInput } from "@/lib/dates";
import type { Day } from "@/lib/days";
import { errorMessage } from "@/lib/errors";
import { capacity, DEFAULT_ESTIMATE_MIN } from "./capacity";
import { CapacityBar } from "./capacity-bar";
import { EstimateSelect } from "./estimate-select";

type PlanDayProps = { today: Day; endOfToday: number };

/** The morning plan: pick today's tasks, estimate them and compare with the hours you have. */
export function PlanDay({ today, endOfToday }: PlanDayProps) {
  const planned = usePlanned(today, today);
  const available = useAvailableMinutes();
  const [query, setQuery] = useState("");
  const candidates = useCandidates(today, endOfToday, query);
  const setPlan = useSetPlan();
  const tasks = planned.data ?? [];
  const error = planned.error ?? candidates.error ?? available.error ?? setPlan.error;

  function plan(task: PlannedTask, changes: Partial<PlannedTask>) {
    setPlan.mutate({
      id: task.id,
      plan: {
        planned_on: task.planned_on,
        scheduled_at: task.scheduled_at,
        duration_min: task.duration_min,
        ...changes,
      },
    });
  }

  return (
    <div className="flex flex-col gap-5">
      {available.data !== undefined && <CapacityBar capacity={capacity(tasks, available.data)} />}
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <section className="flex flex-col gap-2">
        <h3 className="text-label uppercase tracking-wider text-muted-foreground">Today</h3>
        {tasks.length === 0 && (
          <p className="text-sm text-muted-foreground">Nothing planned yet. Add tasks below.</p>
        )}
        <ul className="flex flex-col gap-1">
          {tasks.map((task) => (
            <li key={task.id} className="flex items-center gap-2">
              <TaskLabel task={task} />
              <EstimateSelect
                value={task.duration_min}
                onChange={(duration_min) => plan(task, { duration_min })}
              />
              <Button
                variant="ghost"
                size="icon"
                aria-label={`Take ${task.title} off today`}
                onClick={() => plan(task, { planned_on: null, scheduled_at: null })}
              >
                <X size={14} />
              </Button>
            </li>
          ))}
        </ul>
      </section>
      <section className="flex flex-col gap-2">
        <h3 className="text-label uppercase tracking-wider text-muted-foreground">Add tasks</h3>
        <Input
          placeholder="Search open tasks"
          aria-label="Search open tasks"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        {candidates.data?.length === 0 && (
          <p className="text-sm text-muted-foreground">
            {query.trim() ? "No open task matches." : "Nothing due, in progress or in the Inbox."}
          </p>
        )}
        <ul className="flex max-h-72 flex-col gap-1 overflow-auto">
          {candidates.data?.map((task) => (
            <li key={task.id} className="flex items-center gap-2">
              <TaskLabel task={task} today={today} />
              <Button
                variant="outline"
                size="sm"
                disabled={setPlan.isPending}
                onClick={() =>
                  plan(task, {
                    planned_on: today,
                    scheduled_at: null,
                    duration_min: task.duration_min ?? DEFAULT_ESTIMATE_MIN,
                  })
                }
              >
                <Plus size={14} />
                Today
              </Button>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

function TaskLabel({ task, today }: { task: PlannedTask; today?: Day }) {
  const due = task.due_at === null ? null : toDateInput(task.due_at);
  return (
    <span className="flex min-w-0 flex-1 items-center gap-2 text-sm">
      <span className="truncate">
        {task.title}
        <span className="text-muted-foreground"> · {task.project_name}</span>
      </span>
      {today && due !== null && due <= today && (
        <Badge className={due < today ? "bg-destructive text-primary-foreground" : undefined}>
          {due < today ? "Overdue" : "Due today"}
        </Badge>
      )}
      {today && task.planned_on !== null && task.planned_on < today && <Badge>Left over</Badge>}
      {today && task.status !== "todo" && <Badge>In progress</Badge>}
    </span>
  );
}
