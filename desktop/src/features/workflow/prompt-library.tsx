import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useProjects } from "@/hooks/use-projects";
import { useCopyPrompt, usePromptVariables, usePrompts } from "@/hooks/use-prompts";
import type { Prompt } from "@/lib/api/prompts";
import { errorMessage } from "@/lib/errors";
import { CopyPromptDialog } from "./copy-prompt-dialog";
import { filterPrompts } from "./filter";
import { PromptCard } from "./prompt-card";
import { PromptDialog } from "./prompt-dialog";

const ALL_PROJECTS = { search: null, tag: null, include_archived: true };

export function PromptLibrary() {
  const prompts = usePrompts();
  const projects = useProjects(ALL_PROJECTS);
  const variables = usePromptVariables();
  const copy = useCopyPrompt();
  const [search, setSearch] = useState("");
  const [scope, setScope] = useState("all");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [editing, setEditing] = useState<Prompt | null>(null);
  const [creating, setCreating] = useState(false);
  const [copying, setCopying] = useState<{ prompt: Prompt; variables: string[] } | null>(null);

  const projectList = projects.data ?? [];
  const names = new Map(projectList.map((project) => [project.id, project.name]));
  const visible = filterPrompts(prompts.data ?? [], { search, scope, favoritesOnly });
  const error = prompts.error ?? projects.error ?? variables.error ?? copy.error;

  // a project prompt with nothing to ask is copied right away
  function startCopy(prompt: Prompt) {
    variables.mutate(prompt.id, {
      onSuccess: (unknown) => {
        if (prompt.project_id !== null && unknown.length === 0) {
          copy.mutate({ id: prompt.id, projectId: prompt.project_id, variables: {} });
        } else {
          setCopying({ prompt, variables: unknown });
        }
      },
    });
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2">
        <Input
          type="search"
          placeholder="Search prompts"
          aria-label="Search prompts"
          className="max-w-xs"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <Select
          aria-label="Project"
          value={scope}
          onChange={(event) => setScope(event.target.value)}
        >
          <option value="all">All prompts</option>
          <option value="global">Global</option>
          {projectList.map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </Select>
        <Button
          variant={favoritesOnly ? "outline" : "ghost"}
          aria-pressed={favoritesOnly}
          onClick={() => setFavoritesOnly(!favoritesOnly)}
        >
          Favorites
        </Button>
        {copy.isSuccess && <span className="text-sm text-muted-foreground">Copied</span>}
        <Button className="ml-auto" onClick={() => setCreating(true)}>
          New prompt
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}

      {prompts.isSuccess && visible.length === 0 && (
        <p className="py-12 text-center text-muted-foreground">
          {prompts.data.length === 0
            ? "No prompts yet. Save the ones you reuse."
            : "No prompts match the filters."}
        </p>
      )}
      <div className="grid gap-3 lg:grid-cols-2">
        {visible.map((prompt) => (
          <PromptCard
            key={prompt.id}
            prompt={prompt}
            projectName={prompt.project_id ? (names.get(prompt.project_id) ?? "Project") : null}
            onCopy={startCopy}
            onEdit={setEditing}
          />
        ))}
      </div>

      <PromptDialog
        prompt={editing}
        defaultProjectId={scope === "all" || scope === "global" ? null : scope}
        projects={projectList}
        open={creating || editing !== null}
        onOpenChange={(open) => {
          if (!open) {
            setCreating(false);
            setEditing(null);
          }
        }}
      />
      <CopyPromptDialog
        prompt={copying?.prompt ?? null}
        variables={copying?.variables ?? []}
        projects={projectList}
        onClose={() => setCopying(null)}
      />
    </div>
  );
}
