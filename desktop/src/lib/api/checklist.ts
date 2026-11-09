import { invoke } from "@tauri-apps/api/core";

export type ChecklistItem = {
  id: string;
  created_at: number;
  task_id: string;
  text: string;
  done: boolean;
  order_key: string;
};

export function listChecklist(taskId: string) {
  return invoke<ChecklistItem[]>("checklist_list", { taskId });
}

export function createChecklistItem(taskId: string, text: string) {
  return invoke<ChecklistItem>("checklist_create", { taskId, text });
}

export function updateChecklistItem(id: string, text: string, done: boolean) {
  return invoke<ChecklistItem>("checklist_update", { id, text, done });
}

export function deleteChecklistItem(id: string) {
  return invoke<void>("checklist_delete", { id });
}
