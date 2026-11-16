import { invoke } from "@tauri-apps/api/core";

export type McpCall = {
  id: number;
  at: number;
  tool: string;
  args: string;
  ok: boolean;
  summary: string;
  undone: boolean;
  can_undo: boolean;
};

export type McpStatus = {
  enabled: boolean;
  running: boolean;
  port: number;
  /** only while the server is on */
  token: string | null;
  calls: McpCall[];
};

export type BriefKind = "morning" | "review" | "jobs";
export type BriefSetting = { kind: BriefKind; enabled: boolean };

export function getMcpStatus() {
  return invoke<McpStatus>("mcp_status");
}

export function setMcpEnabled(enabled: boolean) {
  return invoke<McpStatus>("mcp_set_enabled", { enabled });
}

export function newMcpToken() {
  return invoke<McpStatus>("mcp_new_token");
}

export function undoMcpCall(id: number) {
  return invoke<string>("mcp_undo", { id });
}

export function getBriefs() {
  return invoke<BriefSetting[]>("briefs_status");
}

export function setBriefEnabled(kind: BriefKind, enabled: boolean) {
  return invoke<void>("briefs_set_enabled", { kind, enabled });
}

/** Writes the brief now; returns the note's path in the vault. */
export function runBrief(kind: BriefKind) {
  return invoke<string>("briefs_run", { kind });
}
