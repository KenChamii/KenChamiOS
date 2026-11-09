import {
  DndContext,
  DragOverlay,
  PointerSensor,
  closestCorners,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useState } from "react";
import { Card } from "@/components/ui/card";
import { useMoveTask } from "@/hooks/use-tasks";
import { TASK_STATUSES, type Task, type TaskStatus } from "@/lib/api/tasks";
import { cn } from "@/lib/cn";
import { CopyTaskPromptButton } from "@/features/workflow/copy-task-prompt-button";
import { dropTarget, groupByStatus } from "./board";
import { STATUS_LABELS } from "./meta";
import { TaskMetaBadges } from "./task-meta-badges";

type KanbanBoardProps = {
  projectId: string;
  tasks: Task[];
  onSelect: (id: string) => void;
};

export function KanbanBoard({ projectId, tasks, onSelect }: KanbanBoardProps) {
  const move = useMoveTask(projectId);
  const [activeTask, setActiveTask] = useState<Task | null>(null);
  const columns = groupByStatus(tasks);
  // a small drag distance keeps plain clicks free to open the task
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  function handleDragStart(event: DragStartEvent) {
    setActiveTask(tasks.find((task) => task.id === event.active.id) ?? null);
  }

  function handleDragEnd(event: DragEndEvent) {
    setActiveTask(null);
    if (!event.over) return;
    const drop = dropTarget(columns, String(event.active.id), String(event.over.id));
    if (drop) move.mutate({ id: String(event.active.id), ...drop });
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCorners}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={() => setActiveTask(null)}
    >
      <div className="grid grid-cols-4 gap-4">
        {TASK_STATUSES.map((status) => (
          <Column key={status} status={status} tasks={columns[status]} onSelect={onSelect} />
        ))}
      </div>
      <DragOverlay>{activeTask && <TaskCard task={activeTask} />}</DragOverlay>
    </DndContext>
  );
}

type ColumnProps = { status: TaskStatus; tasks: Task[]; onSelect: (id: string) => void };

function Column({ status, tasks, onSelect }: ColumnProps) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  return (
    <section
      ref={setNodeRef}
      className={cn(
        "flex min-h-40 flex-col gap-2 rounded-lg bg-muted p-2",
        isOver && "ring-2 ring-primary",
      )}
    >
      <h3 className="px-1 text-sm font-medium">
        {STATUS_LABELS[status]} <span className="text-muted-foreground">{tasks.length}</span>
      </h3>
      <SortableContext items={tasks.map((task) => task.id)} strategy={verticalListSortingStrategy}>
        {tasks.map((task) => (
          <SortableCard key={task.id} task={task} onSelect={onSelect} />
        ))}
      </SortableContext>
    </section>
  );
}

function SortableCard({ task, onSelect }: { task: Task; onSelect: (id: string) => void }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: task.id,
  });
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={cn(isDragging && "opacity-40")}
      onClick={() => onSelect(task.id)}
      {...attributes}
      {...listeners}
    >
      <TaskCard task={task} />
    </div>
  );
}

function TaskCard({ task }: { task: Task }) {
  return (
    <Card className="flex cursor-grab flex-col gap-2 p-3">
      <div className="flex items-start justify-between gap-1">
        <span className="text-sm">{task.title}</span>
        <CopyTaskPromptButton taskId={task.id} />
      </div>
      <div className="flex flex-wrap gap-1">
        <TaskMetaBadges task={task} />
      </div>
    </Card>
  );
}
