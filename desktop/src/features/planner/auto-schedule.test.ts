import { describe, expect, it } from "vitest";
import type { FocusBlock, PlannedTask } from "@/lib/api/planner";
import { autoSchedule, waitingCount } from "./auto-schedule";
import { slotStart } from "./week-grid";

const DAY = "2026-03-10";
const NEVER = 0;

function task(id: string, overrides: Partial<PlannedTask> = {}): PlannedTask {
  return {
    id,
    title: id,
    project_id: "p",
    project_name: "P",
    project_color: null,
    status: "todo",
    priority: "medium",
    due_at: null,
    recurrence: null,
    planned_on: DAY,
    scheduled_at: null,
    duration_min: 60,
    ...overrides,
  };
}

const starts = (tasks: PlannedTask[], focus: FocusBlock[] = [], now = NEVER) =>
  autoSchedule(tasks, focus, now).placed.map((p) => [p.task.id, p.plan.scheduled_at]);

describe("autoSchedule", () => {
  it("fills the day from the first hour, one task after the other", () => {
    expect(starts([task("a"), task("b")])).toEqual([
      ["a", slotStart(DAY, 0)],
      ["b", slotStart(DAY, 2)],
    ]);
  });

  it("puts the earliest due date first, then the higher priority", () => {
    const tasks = [
      task("late", { due_at: 2000 }),
      task("soon", { due_at: 1000, priority: "low" }),
      task("none-high", { priority: "high" }),
      task("none-low", { priority: "low" }),
    ];
    expect(autoSchedule(tasks, [], NEVER).placed.map((p) => p.task.id)).toEqual([
      "soon",
      "late",
      "none-high",
      "none-low",
    ]);
  });

  it("works around fixed blocks and recorded focus sessions", () => {
    const fixed = task("fixed", { scheduled_at: slotStart(DAY, 0), duration_min: 60 });
    const focus = {
      started_at: slotStart(DAY, 2),
      ended_at: slotStart(DAY, 3),
    } as FocusBlock;
    expect(starts([fixed, task("a")], [focus])).toEqual([["a", slotStart(DAY, 3)]]);
  });

  it("never starts in the past", () => {
    const now = slotStart(DAY, 4) - 1;
    expect(starts([task("a")], [], now)).toEqual([["a", slotStart(DAY, 4)]]);
  });

  it("leaves a task unplaced when the day has no room", () => {
    const result = autoSchedule([task("huge", { duration_min: 24 * 60 })], [], NEVER);
    expect(result.placed).toEqual([]);
    expect(result.unplaced.map((t) => t.id)).toEqual(["huge"]);
  });

  it("ignores finished tasks and tasks without a day", () => {
    const tasks = [task("done", { status: "done" }), task("loose", { planned_on: null })];
    expect(autoSchedule(tasks, [], NEVER)).toEqual({ placed: [], unplaced: [] });
  });

  it("uses the default estimate when a task has none", () => {
    const result = autoSchedule([task("a", { duration_min: null })], [], NEVER);
    expect(result.placed[0].plan.duration_min).toBe(30);
  });
});

describe("waitingCount", () => {
  it("counts planned tasks without a time", () => {
    const tasks = [task("a"), task("b", { scheduled_at: 1 }), task("c", { status: "done" })];
    expect(waitingCount(tasks)).toBe(1);
  });
});
