import type { PlanProposal } from "@/lib/api/agent";

export function countTasks(plan: PlanProposal): number {
  return plan.phases.reduce((total, phase) => total + phase.tasks.length, 0);
}

export function renamePhase(plan: PlanProposal, phase: number, title: string): PlanProposal {
  return {
    phases: plan.phases.map((p, i) => (i === phase ? { ...p, title } : p)),
  };
}

export function renameTask(
  plan: PlanProposal,
  phase: number,
  task: number,
  title: string,
): PlanProposal {
  return {
    phases: plan.phases.map((p, i) =>
      i === phase ? { ...p, tasks: p.tasks.map((t, j) => (j === task ? { ...t, title } : t)) } : p,
    ),
  };
}

/** Removes a task, and its phase when that was the last task in it. */
export function removeTask(plan: PlanProposal, phase: number, task: number): PlanProposal {
  return {
    phases: plan.phases
      .map((p, i) => (i === phase ? { ...p, tasks: p.tasks.filter((_, j) => j !== task) } : p))
      .filter((p) => p.tasks.length > 0),
  };
}

/** True when there is something to create and every title has text. */
export function canCreate(plan: PlanProposal): boolean {
  return (
    plan.phases.length > 0 &&
    plan.phases.every(
      (phase) => phase.title.trim() !== "" && phase.tasks.every((task) => task.title.trim() !== ""),
    )
  );
}
