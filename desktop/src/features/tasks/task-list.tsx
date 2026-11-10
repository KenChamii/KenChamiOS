import { Checkbox } from "@/components/ui/checkbox";
import { useMoveTask } from "@/hooks/use-tasks";
import type { Task } from "@/lib/api/tasks";
import { cn } from "@/lib/cn";
import { stagger } from "@/lib/motion";
import { StartTimerButton } from "@/features/focus/start-timer-button";
import { CopyTaskPromptButton } from "@/features/workflow/copy-task-prompt-button";
import { TaskMetaBadges } from "./task-meta-badges";

type TaskListProps = {
  projectId: string;
  tasks: Task[];
  onSelect: (id: string) => void;
};

export function TaskList({ projectId, tasks, onSelect }: TaskListProps) {
  const move = useMoveTask(projectId);

  return (
    <ul className="flex flex-col divide-y divide-border rounded-2xl bg-card shadow-soft">
      {tasks.map((task, index) => {
        const done = task.status === "done";
        return (
          <li
            key={task.id}
            className="stagger-in flex items-center gap-3 px-4 py-2"
            style={stagger(index)}
          >
            <Checkbox
              aria-label={`Complete ${task.title}`}
              checked={done}
              onCheckedChange={(checked) =>
                move.mutate({ task, status: checked ? "done" : "todo", beforeId: null })
              }
            />
            <button
              type="button"
              onClick={() => onSelect(task.id)}
              className={cn(
                "flex-1 text-left text-sm",
                done && "text-muted-foreground line-through",
              )}
            >
              {task.title}
            </button>
            <TaskMetaBadges task={task} />
            <StartTimerButton taskId={task.id} />
            <CopyTaskPromptButton taskId={task.id} />
          </li>
        );
      })}
    </ul>
  );
}
