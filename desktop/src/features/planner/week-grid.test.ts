import { describe, expect, it } from "vitest";
import {
  HOUR_PX,
  SLOT_COUNT,
  blockLayout,
  dayBounds,
  parseDrop,
  resizedDuration,
  slotId,
  slotStart,
  weekDays,
} from "./week-grid";

const at = (day: string, hour: number, minute = 0) => {
  const [y, m, d] = day.split("-").map(Number);
  return new Date(y, m - 1, d, hour, minute).getTime();
};

describe("week grid", () => {
  it("lists monday to sunday", () => {
    expect(weekDays("2026-11-12")).toEqual([
      "2026-11-09",
      "2026-11-10",
      "2026-11-11",
      "2026-11-12",
      "2026-11-13",
      "2026-11-14",
      "2026-11-15",
    ]);
  });

  it("maps slots to local times from 7:00 in half hours", () => {
    expect(SLOT_COUNT).toBe(30);
    expect(slotStart("2026-11-10", 0)).toBe(at("2026-11-10", 7));
    expect(slotStart("2026-11-10", 3)).toBe(at("2026-11-10", 8, 30));
    expect(dayBounds("2026-11-10")).toEqual({
      from: at("2026-11-10", 0),
      to: at("2026-11-11", 0),
    });
  });

  it("places blocks by start and length and clips them to the grid", () => {
    expect(blockLayout("2026-11-10", at("2026-11-10", 9), at("2026-11-10", 10, 30))).toEqual({
      top: 2 * HOUR_PX,
      height: 1.5 * HOUR_PX,
    });
    expect(blockLayout("2026-11-10", at("2026-11-10", 6), at("2026-11-10", 8))).toEqual({
      top: 0,
      height: HOUR_PX,
    });
    expect(blockLayout("2026-11-10", at("2026-11-10", 22), at("2026-11-10", 23))).toBeNull();
    expect(blockLayout("2026-11-10", at("2026-11-11", 9), at("2026-11-11", 10))).toBeNull();
  });

  it("resizes in quarter hours within bounds", () => {
    expect(resizedDuration(60, HOUR_PX / 2)).toBe(90);
    expect(resizedDuration(60, 5)).toBe(60);
    expect(resizedDuration(30, -HOUR_PX)).toBe(15);
    expect(resizedDuration(1430, HOUR_PX)).toBe(1440);
  });

  it("reads drop targets", () => {
    expect(parseDrop(slotId("2026-11-10", 4))).toEqual({
      kind: "slot",
      day: "2026-11-10",
      slot: 4,
    });
    expect(parseDrop("day:2026-11-10")).toEqual({ kind: "day", day: "2026-11-10" });
    expect(parseDrop("unscheduled")).toEqual({ kind: "unscheduled" });
    expect(parseDrop("slot:2026-11-10:30")).toBeNull();
    expect(parseDrop("slot:2026-11-10:x")).toBeNull();
    expect(parseDrop("todo")).toBeNull();
  });
});
