import { invoke } from "@tauri-apps/api/core";

export const STAGES = ["idea", "plan", "build", "review", "done"] as const;
export type Stage = (typeof STAGES)[number];

export type Project = {
  id: string;
  created_at: number;
  name: string;
  slug: string;
  stage: Stage;
  description: string;
  repo_path: string | null;
  color: string | null;
  tags: string[];
  archived: boolean;
  finished_at: number | null;
};

export type ProjectSummary = Project & {
  tasks_total: number;
  tasks_done: number;
  last_activity_at: number;
};

export type ProjectInput = {
  name: string;
  description: string;
  repo_path: string | null;
  color: string | null;
  tags: string[];
  archived: boolean;
};

export type ProjectFilter = {
  search: string | null;
  tag: string | null;
  include_archived: boolean;
};

export function listProjects(filter: ProjectFilter) {
  return invoke<ProjectSummary[]>("projects_list", { filter });
}

export function getProject(id: string) {
  return invoke<ProjectSummary>("projects_get", { id });
}

export function createProject(input: ProjectInput) {
  return invoke<ProjectSummary>("projects_create", { input });
}

export function updateProject(id: string, input: ProjectInput) {
  return invoke<ProjectSummary>("projects_update", { id, input });
}

export function deleteProject(id: string) {
  return invoke<void>("projects_delete", { id });
}

export function setProjectStage(id: string, stage: Stage) {
  return invoke<ProjectSummary>("project_set_stage", { id, stage });
}
