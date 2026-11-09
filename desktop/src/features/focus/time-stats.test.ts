import { describe, expect, it } from "vitest";
import type { EntryKind, TimeEntry } from "@/lib/api/time-entries";
import { CHART_DAYS, startOfWeek, summarize } from "./time-stats";

// Wednesday 2026-11-11 at noon, local time
const NOW = new Date(2026, 10, 11, 12).getTime();
const at = (day: number, hour = 9) => new Date(2026, 10, day, hour).getTime();

function entry(
  startedAt: number,
  minutes: number,
  taskId: string | null = "t1",
  kind: EntryKind = "focus",
): TimeEntry {
  return {
    id: String(startedAt),
    created_at: startedAt,
    project_id: "p",
    task_id: taskId,
    kind,
    started_at: startedAt,
    ended_at: startedAt + minutes * 60_000,
    duration_s: minutes * 60,
    note: "",
  };
}

describe("startOfWeek", () => {
  it("is the Monday at local midnight", () => {
    const monday = new Date(2026, 10, 9).getTime();
    expect(startOfWeek(NOW)).toBe(monday);
    expect(startOfWeek(new Date(2026, 10, 15, 23).getTime())).toBe(monday);
    expect(startOfWeek(monday)).toBe(monday);
  });
});

describe("summarize", () => {
  it("is empty without entries", () => {
    const summary = summarize([], NOW);
    expect(summary.totalSeconds).toBe(0);
    expect(summary.weekSeconds).toBe(0);
    expect(summary.perTask).toEqual([]);
    expect(summary.days).toHaveLength(CHART_DAYS);
    expect(summary.days.every((day) => day.seconds === 0)).toBe(true);
  });

  it("splits the total, this week and tasks", () => {
    const summary = summarize(
      [
        entry(at(2), 60),
        entry(at(9), 25),
        entry(at(10), 25, "t2"),
        entry(at(11), 30, null, "manual"),
        entry(at(11, 10), 5, "t1", "break"),
      ],
      NOW,
    );

    expect(summary.totalSeconds).toBe(140 * 60);
    expect(summary.weekSeconds).toBe(80 * 60);
    expect(summary.perTask).toEqual([
      { taskId: "t1", seconds: 85 * 60 },
      { taskId: null, seconds: 30 * 60 },
      { taskId: "t2", seconds: 25 * 60 },
    ]);
  });

  it("charts the last days, oldest first, ending today", () => {
    const summary = summarize([entry(at(11), 20), entry(at(11, 11), 10), entry(at(10), 5)], NOW);

    expect(summary.days[CHART_DAYS - 1]).toEqual({ date: "2026-11-11", seconds: 30 * 60 });
    expect(summary.days[CHART_DAYS - 2]).toEqual({ date: "2026-11-10", seconds: 5 * 60 });
    expect(summary.days[0].date).toBe("2026-10-29");
  });
});
