import type { AgentRun, RunStatus } from "@/lib/api/agent";

export const BOARD_COLUMNS = [
  { status: "queued", label: "Queued" },
  { status: "running", label: "Running" },
  { status: "review", label: "Review" },
  { status: "failed", label: "Failed" },
  { status: "done", label: "Done" },
] as const satisfies readonly { status: RunStatus; label: string }[];

export type BoardColumns = Record<(typeof BOARD_COLUMNS)[number]["status"], AgentRun[]>;

/** Runs by column, newest first as given; discarded runs are not shown. */
export function groupRuns(runs: AgentRun[]): BoardColumns {
  const columns: BoardColumns = { queued: [], running: [], review: [], failed: [], done: [] };
  for (const run of runs) {
    if (run.status !== "discarded") columns[run.status].push(run);
  }
  return columns;
}

/** The status line of a run: why it failed, or what the tests said. */
export function runSummary(run: AgentRun): string {
  if (run.status === "failed") return run.error ?? "Failed";
  if (run.status === "review" && run.commit_sha) return "Approved, ready to merge";
  if (run.status === "review" && run.test_exit_code !== null) {
    return run.test_exit_code === 0 ? "Tests passed" : "Tests failed";
  }
  return `${run.turns} turns`;
}

/** Open tasks that no run is working on or waiting to review. */
export function runnableTasks<T extends { id: string; status: string; parent_id: string | null }>(
  tasks: T[],
  runs: Pick<AgentRun, "task_id" | "status">[],
): T[] {
  const taken = new Set(
    runs
      .filter((run) => ["queued", "running", "review"].includes(run.status))
      .map((run) => run.task_id),
  );
  return tasks.filter(
    (task) => task.parent_id === null && task.status === "todo" && !taken.has(task.id),
  );
}
