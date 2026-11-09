export type FirstRunCounts = { projects: number; tasks: number; media: number; devices: number };

export type StepId = "project" | "task" | "media" | "phone";

export type Step = { id: StepId; done: boolean };

/** True for a fresh install: no projects and nothing in the library. */
export function isFirstRun(counts: FirstRunCounts): boolean {
  return counts.projects === 0 && counts.media === 0;
}

export function firstRunSteps(counts: FirstRunCounts): Step[] {
  return [
    { id: "project", done: counts.projects > 0 },
    { id: "task", done: counts.tasks > 0 },
    { id: "media", done: counts.media > 0 },
    { id: "phone", done: counts.devices > 0 },
  ];
}
