import { newTask } from "@/features/tasks/new-task";
import type { ProjectSummary } from "@/lib/api/projects";
import type { TaskCreate } from "@/lib/api/tasks";
import { fromDateInput } from "@/lib/dates";
import type { QuickTask } from "./parse";

/** The project a `#name` points at, by slug or name, ignoring case. */
export function matchProject(name: string, projects: ProjectSummary[]): ProjectSummary | null {
  const wanted = name.toLowerCase();
  return (
    projects.find((project) => project.slug === wanted) ??
    projects.find((project) => project.name.toLowerCase() === wanted) ??
    null
  );
}

export function taskFromQuickAdd(parsed: QuickTask, projectId: string): TaskCreate {
  return {
    ...newTask(projectId, parsed.title, null),
    priority: parsed.priority ?? "medium",
    due_at: parsed.due === null ? null : fromDateInput(parsed.due),
    recurrence: parsed.recurrence,
  };
}
