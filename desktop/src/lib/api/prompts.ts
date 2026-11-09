import { invoke } from "@tauri-apps/api/core";

export type Prompt = {
  id: string;
  created_at: number;
  project_id: string | null;
  title: string;
  body_md: string;
  tags: string[];
  favorite: boolean;
  use_count: number;
  last_used_at: number | null;
};

export type PromptInput = {
  project_id: string | null;
  title: string;
  body_md: string;
  tags: string[];
};

export function listPrompts() {
  return invoke<Prompt[]>("prompts_list");
}

export function createPrompt(input: PromptInput) {
  return invoke<Prompt>("prompts_create", { input });
}

export function updatePrompt(id: string, input: PromptInput) {
  return invoke<Prompt>("prompts_update", { id, input });
}

export function setPromptFavorite(id: string, favorite: boolean) {
  return invoke<Prompt>("prompts_set_favorite", { id, favorite });
}

export function deletePrompt(id: string) {
  return invoke<void>("prompts_delete", { id });
}

export function listPromptVariables(id: string) {
  return invoke<string[]>("prompts_variables", { id });
}

export function copyPrompt(
  id: string,
  projectId: string | null,
  variables: Record<string, string>,
) {
  return invoke<string>("prompts_copy", { id, projectId, variables });
}

export function getTaskPrompt(taskId: string) {
  return invoke<string>("tasks_prompt", { taskId });
}
