import type { FocusBlock, PlannedTask, TaskPlan } from "@/lib/api/planner";
import type { Priority } from "@/lib/api/tasks";
import { DEFAULT_ESTIMATE_MIN } from "./capacity";
import { SLOT_COUNT, slotStart } from "./week-grid";

type Span = { from: number; to: number };

export type Placement = { task: PlannedTask; plan: TaskPlan };
export type AutoSchedule = { placed: Placement[]; unplaced: PlannedTask[] };

const PRIORITY_RANK: Record<Priority, number> = { high: 0, medium: 1, low: 2 };
const MINUTE_MS = 60_000;

/** How many planned tasks are still waiting for a time. */
export function waitingCount(planned: PlannedTask[]): number {
  return planned.filter(needsPlacing).length;
}

function needsPlacing(task: PlannedTask): boolean {
  return task.planned_on !== null && task.scheduled_at === null && task.status !== "done";
}

// earliest due date first, then high priority, then the title so the result is stable
function byRule(a: PlannedTask, b: PlannedTask): number {
  const dueA = a.due_at ?? Number.POSITIVE_INFINITY;
  const dueB = b.due_at ?? Number.POSITIVE_INFINITY;
  if (dueA !== dueB) return dueA < dueB ? -1 : 1;
  const priority = PRIORITY_RANK[a.priority] - PRIORITY_RANK[b.priority];
  return priority !== 0 ? priority : a.title.localeCompare(b.title);
}

function busySpans(planned: PlannedTask[], focus: FocusBlock[]): Span[] {
  const blocks = planned
    .filter((task) => task.scheduled_at !== null && task.status !== "done")
    .map((task) => ({
      from: task.scheduled_at ?? 0,
      to: (task.scheduled_at ?? 0) + (task.duration_min ?? DEFAULT_ESTIMATE_MIN) * MINUTE_MS,
    }));
  return [...blocks, ...focus.map((block) => ({ from: block.started_at, to: block.ended_at }))];
}

function firstFreeSlot(day: string, durationMin: number, busy: Span[], nowMs: number) {
  const gridEnd = slotStart(day, SLOT_COUNT);
  for (let slot = 0; slot < SLOT_COUNT; slot += 1) {
    const from = slotStart(day, slot);
    const to = from + durationMin * MINUTE_MS;
    if (to > gridEnd) return null;
    // a start in the past is no use, and a running block is never moved
    if (from < nowMs) continue;
    if (!busy.some((span) => from < span.to && span.from < to)) return from;
  }
  return null;
}

/**
 * Puts planned tasks that have no time yet into the free hours of their own day: earliest due
 * date first, then priority. Fixed blocks, focus sessions and the past stay where they are.
 */
export function autoSchedule(
  planned: PlannedTask[],
  focus: FocusBlock[],
  nowMs: number,
): AutoSchedule {
  const busy = busySpans(planned, focus);
  const result: AutoSchedule = { placed: [], unplaced: [] };
  for (const task of planned.filter(needsPlacing).sort(byRule)) {
    const day = task.planned_on ?? "";
    const duration = task.duration_min ?? DEFAULT_ESTIMATE_MIN;
    const start = firstFreeSlot(day, duration, busy, nowMs);
    if (start === null) {
      result.unplaced.push(task);
      continue;
    }
    busy.push({ from: start, to: start + duration * MINUTE_MS });
    result.placed.push({
      task,
      plan: { planned_on: day, scheduled_at: start, duration_min: duration },
    });
  }
  return result;
}
