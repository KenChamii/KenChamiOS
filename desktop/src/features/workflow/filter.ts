import type { Prompt } from "@/lib/api/prompts";

/** "all", "global" (no project) or a project id. */
export type PromptScope = string;

export type PromptFilter = { search: string; scope: PromptScope; favoritesOnly: boolean };

export function filterPrompts(prompts: Prompt[], filter: PromptFilter): Prompt[] {
  const needle = filter.search.trim().toLowerCase();
  return prompts.filter((prompt) => {
    if (filter.favoritesOnly && !prompt.favorite) return false;
    if (filter.scope === "global" && prompt.project_id !== null) return false;
    if (filter.scope !== "all" && filter.scope !== "global" && prompt.project_id !== filter.scope) {
      return false;
    }
    if (!needle) return true;
    return [prompt.title, prompt.body_md, ...prompt.tags].some((text) =>
      text.toLowerCase().includes(needle),
    );
  });
}
