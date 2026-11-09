import { invoke } from "@tauri-apps/api/core";

export type EntryKind = "focus" | "break" | "manual";

export type TimeEntry = {
  id: string;
  created_at: number;
  project_id: string;
  task_id: string | null;
  kind: EntryKind;
  started_at: number;
  ended_at: number;
  duration_s: number;
  note: string;
};

export type TimeEntryInput = {
  task_id: string | null;
  started_at: number;
  duration_s: number;
  note: string;
};

export function listTimeEntries(projectId: string) {
  return invoke<TimeEntry[]>("time_entries_list", { projectId });
}

export function createTimeEntry(projectId: string, input: TimeEntryInput) {
  return invoke<TimeEntry>("time_entries_create", { projectId, input });
}

export function updateTimeEntry(id: string, input: TimeEntryInput) {
  return invoke<TimeEntry>("time_entries_update", { id, input });
}

export function deleteTimeEntry(id: string) {
  return invoke<void>("time_entries_delete", { id });
}
