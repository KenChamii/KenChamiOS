import { invoke } from "@tauri-apps/api/core";
import type { MediaType } from "./media";
import type { Priority, TaskStatus } from "./tasks";

export type DueTask = {
  id: string;
  title: string;
  project_id: string;
  project_name: string;
  due_at: number;
  priority: Priority;
  status: TaskStatus;
  recurrence: string | null;
};

export type TimelineGroup = "projects" | "focus" | "media";

export type TimelineItem = {
  id: string;
  kind: string;
  entity_type: string;
  entity_id: string;
  summary: string;
  at: number;
  group: TimelineGroup | null;
};

/** A period in unix ms; `to` and the last bucket boundary are exclusive. */
export type RecapRange = { from: number; to: number; buckets: number[] };

export type ProjectFocus = {
  project_id: string;
  name: string;
  color: string | null;
  seconds: number;
};

export type MediaTypeStat = { media_type: MediaType; units: number; completed: number };

export type RecapView = {
  focus_seconds: number;
  focus_by_bucket: number[];
  focus_by_project: ProjectFocus[];
  media: MediaTypeStat[];
  top_genres: { genre: string; count: number }[];
  stage_changes: number;
  projects_finished: string[];
};

export function listDueTasks(until: number) {
  return invoke<DueTask[]>("dashboard_due_tasks", { until });
}

export function listTimeline(group: TimelineGroup | null, limit: number) {
  return invoke<TimelineItem[]>("dashboard_timeline", { group, limit });
}

export function getRecap(range: RecapRange) {
  return invoke<RecapView>("dashboard_recap", { range });
}
