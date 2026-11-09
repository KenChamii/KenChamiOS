import { invoke } from "@tauri-apps/api/core";

export type AssetKind = "screenshot" | "link";

export type ProjectAsset = {
  id: string;
  created_at: number;
  project_id: string;
  kind: AssetKind;
  path_or_url: string;
  caption: string;
  order_key: string;
};

export function listAssets(projectId: string) {
  return invoke<ProjectAsset[]>("assets_list", { projectId });
}

export function addScreenshot(projectId: string, sourcePath: string, caption: string) {
  return invoke<ProjectAsset>("assets_add_screenshot", { projectId, sourcePath, caption });
}

export function addLink(projectId: string, url: string, caption: string) {
  return invoke<ProjectAsset>("assets_add_link", { projectId, url, caption });
}

export function setAssetCaption(id: string, caption: string) {
  return invoke<ProjectAsset>("assets_set_caption", { id, caption });
}

export function deleteAsset(id: string) {
  return invoke<void>("assets_delete", { id });
}
