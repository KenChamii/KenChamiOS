import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useCreateTask, useDeleteTask, useMoveTask } from "@/hooks/use-tasks";
import type { Task } from "@/lib/api/tasks";
import { cn } from "@/lib/cn";
import { AddItemForm } from "./add-item-form";
import { newTask } from "./new-task";

type SubtasksProps = { parent: Task; subtasks: Task[] };

export function Subtasks({ parent, subtasks }: SubtasksProps) {
  const create = useCreateTask(parent.project_id);
  const move = useMoveTask(parent.project_id);
  const remove = useDeleteTask(parent.project_id);

  return (
    <section className="flex flex-col gap-2">
      <h3 className="text-sm font-medium">Subtasks</h3>
      <ul className="flex flex-col gap-1">
        {subtasks.map((task) => {
          const done = task.status === "done";
          return (
            <li key={task.id} className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={done}
                onChange={() =>
                  move.mutate({ id: task.id, status: done ? "todo" : "done", beforeId: null })
                }
              />
              <span className={cn("flex-1", done && "text-muted-foreground line-through")}>
                {task.title}
              </span>
              <Button
                variant="ghost"
                size="icon"
                aria-label={`Remove ${task.title}`}
                onClick={() => remove.mutate(task.id)}
              >
                <X size={14} />
              </Button>
            </li>
          );
        })}
      </ul>
      <AddItemForm
        placeholder="Add a subtask"
        pending={create.isPending}
        onAdd={(title) => create.mutate(newTask(parent.project_id, title, parent.id))}
      />
    </section>
  );
}
