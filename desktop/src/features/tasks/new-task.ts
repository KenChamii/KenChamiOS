import type { TaskCreate } from "@/lib/api/tasks";

export function newTask(projectId: string, title: string, parentId: string | null): TaskCreate {
  return {
    project_id: projectId,
    parent_id: parentId,
    title,
    body_md: "",
    acceptance_md: "",
    related_files: [],
    priority: "medium",
    due_at: null,
    tags: [],
    recurrence: null,
  };
}
