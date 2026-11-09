import { invoke } from "@tauri-apps/api/core";
import type { Stage } from "./projects";

export type SavePoint = {
  id: string;
  created_at: number;
  project_id: string;
  stage: Stage;
  source: string;
  note: string;
  done_md: string;
  next_md: string;
  session_summary_md: string;
  git_branch: string | null;
  git_commit: string | null;
};

export type SavePointInput = {
  note: string;
  done_md: string;
  next_md: string;
};

export function listSavePoints(projectId: string) {
  return invoke<SavePoint[]>("save_points_list", { projectId });
}

export function createSavePoint(projectId: string, input: SavePointInput) {
  return invoke<SavePoint>("save_points_create", { projectId, input });
}

export function deleteSavePoint(id: string) {
  return invoke<void>("save_points_delete", { id });
}

export function exportSavePoints(projectId: string, path: string) {
  return invoke<void>("save_points_export", { projectId, path });
}
