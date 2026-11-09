import { invoke } from "@tauri-apps/api/core";

export type TemplateKind = "brief" | "task_prompt" | "portfolio";

export type ExportTemplate = {
  id: string;
  created_at: number;
  kind: TemplateKind;
  name: string;
  body: string;
  is_default: boolean;
};

export function listTemplates() {
  return invoke<ExportTemplate[]>("templates_list");
}

export function updateTemplate(id: string, body: string) {
  return invoke<ExportTemplate>("templates_update", { id, body });
}

export function resetTemplate(id: string) {
  return invoke<ExportTemplate>("templates_reset", { id });
}

export function previewTemplate(body: string) {
  return invoke<string>("templates_preview", { body });
}
