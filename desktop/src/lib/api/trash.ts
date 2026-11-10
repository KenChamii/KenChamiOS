import { invoke } from "@tauri-apps/api/core";

export type TrashKind = "project" | "task" | "prompt" | "save_point" | "media" | "habit";

export type TrashItem = {
  kind: TrashKind;
  id: string;
  title: string;
  /** the project name, or the media type */
  detail: string | null;
  deleted_at: number;
};

export function listTrash() {
  return invoke<TrashItem[]>("trash_list");
}

export function restoreFromTrash(kind: TrashKind, id: string) {
  return invoke<void>("trash_restore", { kind, id });
}
