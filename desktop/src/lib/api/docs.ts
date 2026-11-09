import { invoke } from "@tauri-apps/api/core";

export type DocKind = "idea" | "plan" | "adr" | "note";

export type ProjectDoc = {
  id: string;
  created_at: number;
  project_id: string;
  kind: DocKind;
  title: string;
  body_md: string;
};

export function getDoc(projectId: string, kind: DocKind) {
  return invoke<ProjectDoc | null>("docs_get", { projectId, kind });
}

export function upsertDoc(projectId: string, kind: DocKind, title: string, bodyMd: string) {
  return invoke<ProjectDoc>("docs_upsert", { projectId, kind, title, bodyMd });
}
