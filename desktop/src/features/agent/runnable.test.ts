import { describe, expect, it } from "vitest";
import { runnableTasks } from "./runs";

const task = (id: string, status = "todo", parent_id: string | null = null) => ({
  id,
  status,
  parent_id,
});

describe("runnableTasks", () => {
  it("offers open top-level tasks without a live run", () => {
    const tasks = [task("a"), task("b"), task("c", "done"), task("d", "todo", "a"), task("e")];
    const runs = [
      { task_id: "b", status: "running" as const },
      { task_id: "e", status: "failed" as const },
    ];
    expect(runnableTasks(tasks, runs).map((t) => t.id)).toEqual(["a", "e"]);
  });
});
