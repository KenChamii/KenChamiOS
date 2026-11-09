import { invoke } from "@tauri-apps/api/core";

export type DiffLine = { kind: "equal" | "insert" | "delete"; text: string };

export type BriefPreview = {
  target: string;
  file_exists: boolean;
  rendered: string;
  diff: DiffLine[];
  changed: boolean;
};

export function listBriefTargets() {
  return invoke<string[]>("brief_targets");
}

export function getBriefTarget(projectId: string) {
  return invoke<string>("brief_get_target", { projectId });
}

export function setBriefTarget(projectId: string, target: string) {
  return invoke<void>("brief_set_target", { projectId, target });
}

export function renderBrief(projectId: string) {
  return invoke<string>("brief_render", { projectId });
}

export function previewBrief(projectId: string) {
  return invoke<BriefPreview>("brief_preview", { projectId });
}

export function writeBrief(projectId: string) {
  return invoke<void>("brief_write", { projectId });
}
