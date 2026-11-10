import type { HabitCheck } from "@/lib/api/habits";
import type { Day } from "@/lib/days";
import type { TaskStatus } from "@/lib/api/tasks";

type Planned = { planned_on: Day | null; status: TaskStatus };

/** Planned for a day that is over and still not done. */
export function slipped<T extends Planned>(tasks: T[], today: Day): T[] {
  return tasks.filter(
    (task) => task.planned_on !== null && task.planned_on < today && task.status !== "done",
  );
}

export type HabitWeek = { habitId: string; checked: number; days: number; percent: number };

/** Check-ins per habit over the days of the week that have started. */
export function habitWeek(
  habitIds: string[],
  checks: HabitCheck[],
  week: Day[],
  today: Day,
): HabitWeek[] {
  const days = week.filter((day) => day <= today);
  return habitIds.map((habitId) => {
    const checked = new Set(
      checks
        .filter((check) => check.habit_id === habitId && days.includes(check.day))
        .map((check) => check.day),
    ).size;
    const percent = days.length === 0 ? 0 : Math.round((checked / days.length) * 100);
    return { habitId, checked, days: days.length, percent };
  });
}

/** Distinct projects of the tasks, in first-seen order. */
export function touchedProjects<T extends { project_id: string; project_name: string }>(
  tasks: T[],
  skip: string,
): { id: string; name: string }[] {
  const seen = new Map<string, string>();
  for (const task of tasks) {
    if (task.project_id !== skip && !seen.has(task.project_id)) {
      seen.set(task.project_id, task.project_name);
    }
  }
  return [...seen].map(([id, name]) => ({ id, name }));
}
