import { invoke } from "@tauri-apps/api/core";

export type VaultStatus = {
  path: string;
  enabled: boolean;
  last_sync: number | null;
  excluded: string[];
  exists: boolean;
};

export type VaultChange = {
  path: string;
  kind: "added" | "updated" | "conflict";
  direction: "to_vault" | "to_app";
};

export type VaultReport = { changes: VaultChange[]; skipped: number; inbox_imported: number };

export function getVaultStatus() {
  return invoke<VaultStatus>("vault_status");
}

export function setVaultPath(path: string) {
  return invoke<void>("vault_set_path", { path });
}

export function setVaultEnabled(enabled: boolean) {
  return invoke<void>("vault_set_enabled", { enabled });
}

export function setVaultExcluded(names: string[]) {
  return invoke<void>("vault_set_excluded", { names });
}

/** What a sync would do; nothing is written. */
export function previewVault() {
  return invoke<VaultReport>("vault_preview");
}

export function syncVaultNow() {
  return invoke<VaultReport>("vault_sync_now");
}

export type GraphKind = "note" | "project" | "daily" | "tag" | "job" | "company";
export type GraphNode = { id: string; kind: GraphKind; label: string; path: string | null };
export type GraphEdge = { source: string; target: string; kind: string };
export type GraphData = { nodes: GraphNode[]; edges: GraphEdge[] };

/** Re-reads the vault and the app records, then returns every node and link. */
export function getGraph() {
  return invoke<GraphData>("graph_data");
}
