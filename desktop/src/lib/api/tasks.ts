import { invoke } from "@tauri-apps/api/core";

export const TASK_STATUSES = ["todo", "doing", "review", "done"] as const;
export type TaskStatus = (typeof TASK_STATUSES)[number];

export const PRIORITIES = ["low", "medium", "high"] as const;
export type Priority = (typeof PRIORITIES)[number];

export type Task = {
  id: string;
  created_at: number;
  project_id: string;
  parent_id: string | null;
  title: string;
  body_md: string;
  acceptance_md: string;
  related_files: string[];
  status: TaskStatus;
  priority: Priority;
  due_at: number | null;
  order_key: string;
  tags: string[];
  /** e.g. "weekly:2", see lib/recurrence */
  recurrence: string | null;
  /** local day the task is planned for, see lib/api/planner */
  planned_on: string | null;
  scheduled_at: number | null;
  duration_min: number | null;
};

export type TaskCreate = {
  project_id: string;
  parent_id: string | null;
  title: string;
  body_md: string;
  acceptance_md: string;
  related_files: string[];
  priority: Priority;
  due_at: number | null;
  tags: string[];
  recurrence: string | null;
};

export type TaskUpdate = Omit<TaskCreate, "project_id" | "parent_id">;

export function listTasks(projectId: string) {
  return invoke<Task[]>("tasks_list", { projectId });
}

export function createTask(input: TaskCreate) {
  return invoke<Task>("tasks_create", { input });
}

export function updateTask(id: string, input: TaskUpdate) {
  return invoke<Task>("tasks_update", { id, input });
}

export function deleteTask(id: string) {
  return invoke<void>("tasks_delete", { id });
}

/** The moved task, and the next occurrence when completing a repeating task added one. */
export type TaskMoved = { task: Task; next: Task | null };

export function moveTask(
  id: string,
  status: TaskStatus,
  beforeId: string | null,
  nextDueAt: number | null,
) {
  return invoke<TaskMoved>("tasks_move", { id, status, beforeId, nextDueAt });
}

export function moveTaskToProject(id: string, projectId: string) {
  return invoke<Task>("tasks_move_to_project", { id, projectId });
}
