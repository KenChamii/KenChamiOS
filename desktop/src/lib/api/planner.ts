import { invoke } from "@tauri-apps/api/core";
import type { Day } from "@/lib/days";
import type { Priority, Task, TaskStatus } from "./tasks";

/** When a task is planned; a block start always comes with its day. */
export type TaskPlan = {
  planned_on: Day | null;
  scheduled_at: number | null;
  duration_min: number | null;
};

export type PlannedTask = TaskPlan & {
  id: string;
  title: string;
  project_id: string;
  project_name: string;
  project_color: string | null;
  status: TaskStatus;
  priority: Priority;
  due_at: number | null;
  recurrence: string | null;
};

/** A recorded focus or manual session. */
export type FocusBlock = {
  id: string;
  project_id: string;
  project_name: string;
  project_color: string | null;
  task_title: string | null;
  started_at: number;
  ended_at: number;
};

export const NO_PLAN: TaskPlan = { planned_on: null, scheduled_at: null, duration_min: null };

export function setTaskPlan(id: string, plan: TaskPlan) {
  return invoke<Task>("planner_set_plan", { id, plan });
}

/** Tasks planned from `from` to `to`, both days included. */
export function listPlanned(from: Day, to: Day) {
  return invoke<PlannedTask[]>("planner_planned", { from, to });
}

/** Suggestions to plan today, or open tasks matching `query`. */
export function listCandidates(today: Day, until: number, query: string | null) {
  return invoke<PlannedTask[]>("planner_candidates", { today, until, query });
}

/** Tasks completed in `[from, to)`. */
export function listCompleted(from: number, to: number) {
  return invoke<PlannedTask[]>("planner_completed", { from, to });
}

export function listFocusBlocks(from: number, to: number) {
  return invoke<FocusBlock[]>("planner_focus_blocks", { from, to });
}

export function getAvailableMinutes() {
  return invoke<number>("planner_available_get");
}

export function setAvailableMinutes(minutes: number) {
  return invoke<number>("planner_available_set", { minutes });
}
