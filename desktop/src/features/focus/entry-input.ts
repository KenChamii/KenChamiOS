import type { TimeEntry, TimeEntryInput } from "@/lib/api/time-entries";
import { fromDateInput, toDateInput } from "@/lib/dates";

export type EntryDraft = { date: string; minutes: string; taskId: string; note: string };

export function draftOf(entry: TimeEntry | null, now: number): EntryDraft {
  if (!entry) return { date: toDateInput(now), minutes: "30", taskId: "", note: "" };
  return {
    date: toDateInput(entry.started_at),
    minutes: String(Math.max(1, Math.round(entry.duration_s / 60))),
    taskId: entry.task_id ?? "",
    note: entry.note,
  };
}

/** Null while the date or minutes are not usable. */
export function inputOf(draft: EntryDraft, original: TimeEntry | null): TimeEntryInput | null {
  const minutes = Number(draft.minutes);
  const day = fromDateInput(draft.date);
  if (day === null || !Number.isInteger(minutes) || minutes < 1 || minutes > 24 * 60) return null;
  // keep the time of day when the date was not changed
  const unchanged = original !== null && toDateInput(original.started_at) === draft.date;
  return {
    task_id: draft.taskId || null,
    started_at: unchanged ? original.started_at : day,
    duration_s: minutes * 60,
    note: draft.note,
  };
}
