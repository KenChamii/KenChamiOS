import { invoke } from "@tauri-apps/api/core";
import type { Task } from "./tasks";

export type PlanTask = { title: string; body: string; acceptance: string };
export type PlanPhase = { title: string; tasks: PlanTask[] };
export type PlanProposal = { phases: PlanPhase[] };

/** Asks Claude to split the plan doc into phases and tasks. Creates nothing. */
export function previewPlan(projectId: string) {
  return invoke<PlanProposal>("agent_plan_preview", { projectId });
}

export function createPlanTasks(projectId: string, proposal: PlanProposal) {
  return invoke<Task[]>("agent_plan_create", { projectId, proposal });
}

export const RUN_STATUSES = ["queued", "running", "review", "done", "failed", "discarded"] as const;
export type RunStatus = (typeof RUN_STATUSES)[number];

export type AgentRun = {
  id: string;
  created_at: number;
  updated_at: number;
  project_id: string;
  task_id: string;
  status: RunStatus;
  branch: string | null;
  worktree_path: string | null;
  base_branch: string | null;
  base_commit: string | null;
  commit_sha: string | null;
  feedback: string;
  error: string | null;
  turns: number;
  test_exit_code: number | null;
  log_path: string;
  started_at: number | null;
  finished_at: number | null;
  task_title: string;
};

export type StreamEvent =
  | { kind: "init"; model: string; tools: string[] }
  | { kind: "text"; message_id: string; text: string }
  | { kind: "tool_use"; message_id: string; name: string; input: string }
  | { kind: "tool_result"; content: string; is_error: boolean }
  | { kind: "result"; is_error: boolean; text: string; turns: number; cost_usd: number | null }
  | { kind: "raw"; text: string };

export const AGENT_TOOLS = ["Read", "Edit", "Write", "Glob", "Grep", "Bash"] as const;
export type PermissionMode = "dontAsk" | "acceptEdits";

export type AgentSettings = {
  rules: string;
  test_command: string;
  tools: string[];
  permission_mode: PermissionMode;
  timeout_min: number;
  max_turns: number;
};

export type FileChange = { path: string; status: "added" | "deleted" | "renamed" | "modified" };
export type SecretHit = { rule: string; path: string };
export type RunDiff = {
  files: FileChange[];
  patch: string;
  truncated: boolean;
  secret: SecretHit | null;
};
export type RunReview = { diff: RunDiff; test_output: string | null };

export function getAgentSettings(projectId: string) {
  return invoke<AgentSettings>("agent_settings_get", { projectId });
}

export function setAgentSettings(projectId: string, settings: AgentSettings) {
  return invoke<AgentSettings>("agent_settings_set", { projectId, settings });
}

export function listRuns(projectId: string) {
  return invoke<AgentRun[]>("agent_runs_list", { projectId });
}

export function startRun(taskId: string) {
  return invoke<AgentRun>("agent_run_start", { taskId });
}

export function getRunLog(runId: string) {
  return invoke<StreamEvent[]>("agent_run_log", { runId });
}

export function getRunReview(runId: string) {
  return invoke<RunReview>("agent_run_review", { runId });
}

export function approveRun(runId: string, message: string) {
  return invoke<AgentRun>("agent_run_approve", { runId, message });
}

export function mergeRun(runId: string) {
  return invoke<AgentRun>("agent_run_merge", { runId });
}

export function requeueRun(runId: string, feedback: string) {
  return invoke<AgentRun>("agent_run_requeue", { runId, feedback });
}

export function stopRun(runId: string) {
  return invoke<void>("agent_run_stop", { runId });
}

export function discardRun(runId: string) {
  return invoke<AgentRun>("agent_run_discard", { runId });
}
