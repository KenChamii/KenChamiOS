import { invoke } from "@tauri-apps/api/core";

export type ExportResult = { output_dir: string; screenshots: number };

export function renderPortfolio(projectId: string) {
  return invoke<string>("portfolio_render", { projectId });
}

export function exportPortfolio(projectId: string, folder: string) {
  return invoke<ExportResult>("portfolio_export", { projectId, folder });
}
