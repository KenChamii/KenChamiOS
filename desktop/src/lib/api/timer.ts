import { invoke } from "@tauri-apps/api/core";

export type Phase = "focus" | "break" | "long_break";
export type TimerStatus = "running" | "paused";

export type TimerView = {
  project_id: string;
  task_id: string;
  phase: Phase;
  status: TimerStatus;
  round: number;
  length_s: number;
  remaining_s: number;
};

export type TimerSettings = {
  focus_min: number;
  break_min: number;
  long_break_min: number;
};

export const TIMER_EVENT = "timer://tick";

export function getTimer() {
  return invoke<TimerView | null>("timer_get");
}

export function startTimer(taskId: string) {
  return invoke<TimerView>("timer_start", { taskId });
}

export function pauseTimer() {
  return invoke<TimerView | null>("timer_pause");
}

export function resumeTimer() {
  return invoke<TimerView | null>("timer_resume");
}

export function stopTimer() {
  return invoke<void>("timer_stop");
}

export function getTimerSettings() {
  return invoke<TimerSettings>("timer_settings_get");
}

export function setTimerSettings(settings: TimerSettings) {
  return invoke<TimerSettings>("timer_settings_set", { settings });
}
