import { invoke } from "@tauri-apps/api/core";
import type { SavePoint } from "./save-points";

export type ParsedNote = {
  note: string;
  done: string[];
  next: string[];
  changed_files: string[];
};

export type DoneMatch = { item: string; task_id: string | null };

export type NotePreview = { parsed: ParsedNote; done_matches: DoneMatch[] };

export type SessionConfirm = {
  project_id: string;
  note: string;
  summary_md: string;
  done: string[];
  next: string[];
  changed_files: string[];
  done_task_ids: string[];
  new_task_titles: string[];
};

export function previewSessionNote(projectId: string, text: string) {
  return invoke<NotePreview>("session_note_preview", { projectId, text });
}

export function confirmSessionNote(input: SessionConfirm) {
  return invoke<SavePoint>("session_note_confirm", { input });
}
