import { projectPath } from "@/features/projects/tabs";
import type { SearchHit } from "@/lib/api/search";
import { INBOX_ID } from "@/lib/inbox";

/** Where a search hit opens. */
export function hitPath(hit: SearchHit): string {
  const projectId = hit.project_id;
  if (hit.kind === "task" && projectId === INBOX_ID) return "/inbox";
  if (projectId === null) return "/workflow";
  switch (hit.kind) {
    case "task":
      return projectPath(projectId, "tasks");
    case "idea":
      return projectPath(projectId, "idea");
    case "plan":
      return projectPath(projectId, "plan");
    case "prompt":
      return projectPath(projectId, "prompts");
    case "save_point":
      return projectPath(projectId, "save-points");
  }
}
