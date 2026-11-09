import { useMoveTask } from "@/hooks/use-tasks";
import type { Task } from "@/lib/api/tasks";
import { cn } from "@/lib/cn";
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
    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border bg-card">
      {tasks.map((task) => {
        const done = task.status === "done";
        return (
          <li key={task.id} className="flex items-center gap-3 px-4 py-2">
            <input
              type="checkbox"
              aria-label={`Complete ${task.title}`}
              checked={done}
              onChange={() =>
                move.mutate({ id: task.id, status: done ? "todo" : "done", beforeId: null })
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
