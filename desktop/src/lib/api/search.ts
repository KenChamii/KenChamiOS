import { invoke } from "@tauri-apps/api/core";

export type SearchKind = "task" | "idea" | "plan" | "prompt" | "save_point";

export type SearchHit = {
  kind: SearchKind;
  id: string;
  project_id: string | null;
  project_name: string | null;
  title: string;
  /** words around the match in the body; empty when only the title matched */
  snippet: string;
};

export function searchAll(query: string) {
  return invoke<SearchHit[]>("search_all", { query });
}
