import { describe, expect, it } from "vitest";
import type { Task, TaskStatus } from "@/lib/api/tasks";
import { dropTarget, groupByStatus } from "./board";

function task(id: string, status: TaskStatus, parentId: string | null = null): Task {
  return {
    id,
    created_at: 0,
    project_id: "p",
    parent_id: parentId,
    title: id,
    body_md: "",
    acceptance_md: "",
    related_files: [],
    status,
    priority: "medium",
    due_at: null,
    order_key: id,
    tags: [],
  };
}

const columns = groupByStatus([
  task("a", "todo"),
  task("b", "todo"),
  task("c", "todo"),
  task("d", "doing"),
]);

describe("groupByStatus", () => {
  it("groups top-level tasks and skips subtasks", () => {
    const grouped = groupByStatus([task("a", "todo"), task("s", "todo", "a"), task("d", "done")]);
    expect(grouped.todo.map((t) => t.id)).toEqual(["a"]);
    expect(grouped.done.map((t) => t.id)).toEqual(["d"]);
    expect(grouped.doing).toEqual([]);
  });
});

describe("dropTarget", () => {
  it("drops on a column at the end", () => {
    expect(dropTarget(columns, "a", "review")).toEqual({ status: "review", beforeId: null });
  });

  it("goes before the task it is dropped on when moving up", () => {
    expect(dropTarget(columns, "c", "a")).toEqual({ status: "todo", beforeId: "a" });
  });

  it("goes after the task it is dropped on when moving down", () => {
    expect(dropTarget(columns, "a", "b")).toEqual({ status: "todo", beforeId: "c" });
  });

  it("goes last when dropped on the last task of its column", () => {
    expect(dropTarget(columns, "a", "c")).toEqual({ status: "todo", beforeId: null });
  });

  it("goes before the task in another column", () => {
    expect(dropTarget(columns, "a", "d")).toEqual({ status: "doing", beforeId: "d" });
  });

  it("ignores a drop on itself or on unknown ids", () => {
    expect(dropTarget(columns, "a", "a")).toBeNull();
    expect(dropTarget(columns, "a", "zzz")).toBeNull();
  });
});
