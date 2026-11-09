import { TASK_STATUSES, type Task, type TaskStatus } from "@/lib/api/tasks";

export type Columns = Record<TaskStatus, Task[]>;

/** Splits top-level tasks into board columns, keeping the given order. */
export function groupByStatus(tasks: Task[]): Columns {
  const columns: Columns = { todo: [], doing: [], review: [], done: [] };
  for (const task of tasks) {
    if (task.parent_id === null) columns[task.status].push(task);
  }
  return columns;
}

function isStatus(id: string): id is TaskStatus {
  return (TASK_STATUSES as readonly string[]).includes(id);
}

type Drop = { status: TaskStatus; beforeId: string | null };

/** Where a dragged task lands, as a status and the task it goes before (null = end). */
export function dropTarget(columns: Columns, activeId: string, overId: string): Drop | null {
  if (isStatus(overId)) return { status: overId, beforeId: null };
  if (activeId === overId) return null;

  const locate = (id: string) => {
    for (const status of TASK_STATUSES) {
      const index = columns[status].findIndex((task) => task.id === id);
      if (index !== -1) return { status, index };
    }
    return null;
  };
  const active = locate(activeId);
  const over = locate(overId);
  if (!active || !over) return null;

  const column = columns[over.status];
  const movingDown = active.status === over.status && active.index < over.index;
  const beforeId = movingDown ? (column[over.index + 1]?.id ?? null) : overId;
  return { status: over.status, beforeId };
}
