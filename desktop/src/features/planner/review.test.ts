import { describe, expect, it } from "vitest";
import { habitWeek, slipped, touchedProjects } from "./review";

const week = [
  "2026-11-09",
  "2026-11-10",
  "2026-11-11",
  "2026-11-12",
  "2026-11-13",
  "2026-11-14",
  "2026-11-15",
];

describe("weekly review", () => {
  it("counts open tasks planned before today as slipped", () => {
    const tasks = [
      { id: "a", planned_on: "2026-11-09", status: "todo" as const },
      { id: "b", planned_on: "2026-11-09", status: "done" as const },
      { id: "c", planned_on: "2026-11-11", status: "doing" as const },
      { id: "d", planned_on: "2026-11-12", status: "todo" as const },
      { id: "e", planned_on: null, status: "todo" as const },
    ];
    expect(slipped(tasks, "2026-11-12").map((task) => task.id)).toEqual(["a", "c"]);
  });

  it("rates habits over the days of the week so far", () => {
    const checks = [
      { habit_id: "read", day: "2026-11-09" },
      { habit_id: "read", day: "2026-11-10" },
      { habit_id: "read", day: "2026-11-14" },
      { habit_id: "run", day: "2026-11-11" },
    ];
    expect(habitWeek(["read", "run", "new"], checks, week, "2026-11-12")).toEqual([
      { habitId: "read", checked: 2, days: 4, percent: 50 },
      { habitId: "run", checked: 1, days: 4, percent: 25 },
      { habitId: "new", checked: 0, days: 4, percent: 0 },
    ]);
  });

  it("rates a past week over all seven days and a future week as empty", () => {
    const checks = [{ habit_id: "read", day: "2026-11-15" }];
    expect(habitWeek(["read"], checks, week, "2026-12-01")[0]).toMatchObject({
      days: 7,
      percent: 14,
    });
    expect(habitWeek(["read"], checks, week, "2026-11-01")[0]).toMatchObject({
      days: 0,
      percent: 0,
    });
  });

  it("lists each touched project once and skips the inbox", () => {
    const tasks = [
      { project_id: "p1", project_name: "Alpha" },
      { project_id: "inbox", project_name: "Inbox" },
      { project_id: "p2", project_name: "Beta" },
      { project_id: "p1", project_name: "Alpha" },
    ];
    expect(touchedProjects(tasks, "inbox")).toEqual([
      { id: "p1", name: "Alpha" },
      { id: "p2", name: "Beta" },
    ]);
  });
});
