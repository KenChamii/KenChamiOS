import { describe, expect, it } from "vitest";
import type { TimeEntry } from "@/lib/api/time-entries";
import { draftOf, inputOf } from "./entry-input";

const NOON = new Date(2026, 10, 11, 12).getTime();

const saved: TimeEntry = {
  id: "e",
  created_at: 0,
  project_id: "p",
  task_id: "t1",
  kind: "focus",
  started_at: NOON,
  ended_at: NOON + 25 * 60_000,
  duration_s: 25 * 60,
  note: "pairing",
};

describe("draftOf", () => {
  it("starts a new entry today", () => {
    expect(draftOf(null, NOON)).toEqual({
      date: "2026-11-11",
      minutes: "30",
      taskId: "",
      note: "",
    });
  });

  it("fills the form from an entry", () => {
    expect(draftOf(saved, 0)).toEqual({
      date: "2026-11-11",
      minutes: "25",
      taskId: "t1",
      note: "pairing",
    });
  });
});

describe("inputOf", () => {
  it("starts a new entry at the chosen day", () => {
    const input = inputOf({ date: "2026-11-10", minutes: "45", taskId: "", note: "x" }, null);
    expect(input).toEqual({
      task_id: null,
      started_at: new Date(2026, 10, 10).getTime(),
      duration_s: 45 * 60,
      note: "x",
    });
  });

  it("keeps the time of day while the date stays", () => {
    expect(inputOf(draftOf(saved, 0), saved)?.started_at).toBe(NOON);
  });

  it("moves an edited entry to the new day", () => {
    const input = inputOf({ ...draftOf(saved, 0), date: "2026-11-09" }, saved);
    expect(input?.started_at).toBe(new Date(2026, 10, 9).getTime());
  });

  it.each(["", "0", "1.5", "abc", "1441"])("rejects %j minutes", (minutes) => {
    expect(inputOf({ date: "2026-11-10", minutes, taskId: "", note: "" }, null)).toBeNull();
  });

  it("rejects a missing date", () => {
    expect(inputOf({ date: "", minutes: "10", taskId: "", note: "" }, null)).toBeNull();
  });
});
