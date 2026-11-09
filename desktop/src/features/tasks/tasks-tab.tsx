import { ListTodo } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { useCreateTask, useTasks } from "@/hooks/use-tasks";
import { errorMessage } from "@/lib/errors";
import { useOverlays } from "@/stores/overlays";
import { AddItemForm } from "./add-item-form";
import { KanbanBoard } from "./kanban-board";
import { newTask } from "./new-task";
import { TaskDrawer } from "./task-drawer";
import { TaskList } from "./task-list";

type View = "list" | "board";

export function TasksTab({ projectId }: { projectId: string }) {
  const tasks = useTasks(projectId);
  const create = useCreateTask(projectId);
  const [view, setView] = useState<View>("list");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const openQuickAdd = useOverlays((state) => state.openQuickAdd);

  const all = tasks.data ?? [];
  const topLevel = all.filter((task) => task.parent_id === null);
  const selected = all.find((task) => task.id === selectedId) ?? null;
  const subtasks = selectedId ? all.filter((task) => task.parent_id === selectedId) : [];

  if (tasks.error) return <p className="text-sm text-destructive">{errorMessage(tasks.error)}</p>;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <div className="max-w-md flex-1">
          <AddItemForm
            placeholder="Add a task"
            pending={create.isPending}
            onAdd={(title) => create.mutate(newTask(projectId, title, null))}
          />
        </div>
        <div className="ml-auto flex gap-1" role="group" aria-label="View">
          <Button
            size="sm"
            variant={view === "list" ? "outline" : "ghost"}
            aria-pressed={view === "list"}
            onClick={() => setView("list")}
          >
            List
          </Button>
          <Button
            size="sm"
            variant={view === "board" ? "outline" : "ghost"}
            aria-pressed={view === "board"}
            onClick={() => setView("board")}
          >
            Board
          </Button>
        </div>
      </div>

      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}

      {tasks.isSuccess && topLevel.length === 0 ? (
        <EmptyState
          icon={ListTodo}
          area="projects"
          line="No tasks yet. What is the next small step?"
          action={<Button onClick={() => openQuickAdd("task")}>Add a task</Button>}
        />
      ) : view === "list" ? (
        <TaskList projectId={projectId} tasks={topLevel} onSelect={setSelectedId} />
      ) : (
        <KanbanBoard projectId={projectId} tasks={topLevel} onSelect={setSelectedId} />
      )}

      <TaskDrawer task={selected} subtasks={subtasks} onClose={() => setSelectedId(null)} />
    </div>
  );
}
