import { addDays, mondayOf, type Day } from "@/lib/days";

export const FIRST_HOUR = 7;
export const LAST_HOUR = 22;
export const SLOT_MIN = 30;
export const HOUR_PX = 48;
export const PX_PER_MIN = HOUR_PX / 60;
export const SLOT_COUNT = ((LAST_HOUR - FIRST_HOUR) * 60) / SLOT_MIN;
const GRID_MIN = (LAST_HOUR - FIRST_HOUR) * 60;
const RESIZE_STEP_MIN = 15;
const MAX_DURATION_MIN = 24 * 60;

/** Monday to Sunday of the week holding `day`. */
export function weekDays(day: Day): Day[] {
  const monday = mondayOf(day);
  return Array.from({ length: 7 }, (_, index) => addDays(monday, index));
}

function localDate(day: Day, minutes = 0): Date {
  const [year, month, date] = day.split("-").map(Number);
  return new Date(year, month - 1, date, 0, minutes);
}

/** Local unix ms where a slot of `day` starts. */
export function slotStart(day: Day, slot: number): number {
  return localDate(day, FIRST_HOUR * 60 + slot * SLOT_MIN).getTime();
}

/** Local midnight of `day` and of the day after, as a `[from, to)` range in unix ms. */
export function dayBounds(day: Day): { from: number; to: number } {
  return { from: localDate(day).getTime(), to: localDate(addDays(day, 1)).getTime() };
}

/** Where a block sits in a day column, in px from the first hour; null when it is not visible. */
export function blockLayout(
  day: Day,
  startMs: number,
  endMs: number,
): { top: number; height: number } | null {
  const gridStart = slotStart(day, 0);
  const top = Math.max(0, (startMs - gridStart) / 60_000);
  const bottom = Math.min(GRID_MIN, (endMs - gridStart) / 60_000);
  if (bottom <= top) return null;
  return { top: top * PX_PER_MIN, height: (bottom - top) * PX_PER_MIN };
}

/** A block's new length after dragging its bottom edge by `deltaPx`, in 15 minute steps. */
export function resizedDuration(durationMin: number, deltaPx: number): number {
  const raw = durationMin + deltaPx / PX_PER_MIN;
  const stepped = Math.round(raw / RESIZE_STEP_MIN) * RESIZE_STEP_MIN;
  return Math.min(MAX_DURATION_MIN, Math.max(RESIZE_STEP_MIN, stepped));
}

export type Drop =
  { kind: "slot"; day: Day; slot: number } | { kind: "day"; day: Day } | { kind: "unscheduled" };

export const slotId = (day: Day, slot: number) => `slot:${day}:${slot}`;
export const dayId = (day: Day) => `day:${day}`;
export const UNSCHEDULED_ID = "unscheduled";

/** What a droppable id points at, or null for an unknown id. */
export function parseDrop(id: string): Drop | null {
  if (id === UNSCHEDULED_ID) return { kind: "unscheduled" };
  const [kind, day, slot] = id.split(":");
  if (kind === "day" && day) return { kind: "day", day };
  const index = Number(slot);
  if (kind === "slot" && day && Number.isInteger(index) && index >= 0 && index < SLOT_COUNT) {
    return { kind: "slot", day, slot: index };
  }
  return null;
}
