import { describe, expect, it } from "vitest";
import type { AgentRun } from "@/lib/api/agent";
import { groupRuns, runSummary } from "./runs";

function run(overrides: Partial<AgentRun>): AgentRun {
  return {
    id: "r",
    created_at: 0,
    updated_at: 0,
    project_id: "p",
    task_id: "t",
    status: "queued",
    branch: null,
    worktree_path: null,
    base_branch: null,
    base_commit: null,
    commit_sha: null,
    feedback: "",
    error: null,
    turns: 0,
    test_exit_code: null,
    log_path: "",
    started_at: null,
    finished_at: null,
    task_title: "Task",
    ...overrides,
  };
}

describe("groupRuns", () => {
  it("puts each run in its column and hides discarded ones", () => {
    const columns = groupRuns([
      run({ id: "a", status: "queued" }),
      run({ id: "b", status: "review" }),
      run({ id: "c", status: "discarded" }),
      run({ id: "d", status: "failed" }),
    ]);
    expect(columns.queued.map((r) => r.id)).toEqual(["a"]);
    expect(columns.review.map((r) => r.id)).toEqual(["b"]);
    expect(columns.failed.map((r) => r.id)).toEqual(["d"]);
    expect(columns.running).toEqual([]);
    expect(columns.done).toEqual([]);
  });
});

describe("runSummary", () => {
  it("shows the failure reason", () => {
    expect(runSummary(run({ status: "failed", error: "stopped" }))).toBe("stopped");
  });

  it("shows the test result in review and the approval after", () => {
    expect(runSummary(run({ status: "review", test_exit_code: 0 }))).toBe("Tests passed");
    expect(runSummary(run({ status: "review", test_exit_code: 1 }))).toBe("Tests failed");
    expect(runSummary(run({ status: "review", commit_sha: "abc" }))).toBe(
      "Approved, ready to merge",
    );
  });

  it("falls back to the turn count", () => {
    expect(runSummary(run({ status: "running", turns: 4 }))).toBe("4 turns");
  });
});
