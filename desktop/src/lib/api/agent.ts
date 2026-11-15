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
