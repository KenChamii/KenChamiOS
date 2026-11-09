import { useNavigate } from "react-router-dom";
import { useSearch } from "@/hooks/use-search";
import type { SearchHit, SearchKind } from "@/lib/api/search";
import { INBOX_ID } from "@/lib/inbox";
import { useOverlays } from "@/stores/overlays";
import type { Command } from "./filter";
import { hitPath } from "./hit-path";

const GROUPS: Record<SearchKind, string> = {
  task: "Tasks",
  idea: "Ideas",
  plan: "Plans",
  prompt: "Prompts",
  save_point: "Save points",
};

function hitLabel(hit: SearchHit): string {
  const where = hit.project_name && hit.project_id !== INBOX_ID ? ` · ${hit.project_name}` : "";
  const snippet = hit.snippet ? ` — ${hit.snippet}` : "";
  return `${hit.title}${where}${snippet}`;
}

/** Commands that depend on the typed text: add it as a task, and full-text search hits. */
export function useQueryCommands(query: string): Command[] {
  const navigate = useNavigate();
  const openQuickAdd = useOverlays((state) => state.openQuickAdd);
  const text = query.trim();
  const search = useSearch(text);
  if (!text) return [];

  const addTask: Command = {
    id: "query:add-task",
    label: `Add task: ${text}`,
    group: "Create",
    run: () => openQuickAdd("task", text),
  };
  const hits = text.length >= 2 ? (search.data ?? []) : [];
  return [
    ...hits.map((hit) => ({
      id: `search:${hit.kind}:${hit.id}`,
      label: hitLabel(hit),
      group: GROUPS[hit.kind],
      run: () => navigate(hitPath(hit)),
    })),
    addTask,
  ];
}
