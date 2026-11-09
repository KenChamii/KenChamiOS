import { invoke } from "@tauri-apps/api/core";
import type { Stage } from "./projects";

export type IdleProject = {
  project_id: string;
  name: string;
  stage: Stage;
  idle_days: number;
  last_save_point_at: number | null;
};

export function listIdleProjects() {
  return invoke<IdleProject[]>("reminders_idle_projects");
}

export function snoozeProject(projectId: string) {
  return invoke<void>("reminders_snooze", { projectId });
}

export function getIdleDays() {
  return invoke<number>("reminders_idle_days_get");
}

export function setIdleDays(days: number) {
  return invoke<number>("reminders_idle_days_set", { days });
}
