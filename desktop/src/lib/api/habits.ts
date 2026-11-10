import { invoke } from "@tauri-apps/api/core";
import type { Day } from "@/lib/days";

export type Habit = {
  id: string;
  name: string;
  /** days in a row up to today, or up to yesterday while today is still open */
  streak: number;
  checked_today: boolean;
  /** the last seven days, oldest first */
  recent: boolean[];
};

export function listHabits(today: Day) {
  return invoke<Habit[]>("habits_list", { today });
}

export function createHabit(name: string) {
  return invoke<string>("habits_create", { name });
}

export function deleteHabit(id: string) {
  return invoke<void>("habits_delete", { id });
}

export function toggleHabit(id: string, day: Day) {
  return invoke<boolean>("habits_toggle", { id, day });
}
