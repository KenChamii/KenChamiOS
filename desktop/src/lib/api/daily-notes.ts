import { invoke } from "@tauri-apps/api/core";
import type { Day } from "@/lib/days";

export type DailyNote = { id: string; day: Day; body: string };

export function getDailyNote(day: Day) {
  return invoke<DailyNote | null>("daily_notes_get", { day });
}

export function saveDailyNote(day: Day, body: string) {
  return invoke<DailyNote>("daily_notes_save", { day, body });
}

/** Adds `text` below the day's note. */
export function appendDailyNote(day: Day, text: string) {
  return invoke<DailyNote>("daily_notes_append", { day, text });
}

/** Days from `from` to `to` whose note has text. */
export function listNoteDays(from: Day, to: Day) {
  return invoke<Day[]>("daily_notes_days", { from, to });
}
