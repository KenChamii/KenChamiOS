import { MessageSquareText } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Chip } from "@/components/ui/chips";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useProjects } from "@/hooks/use-projects";
import { useCopyPrompt, usePromptVariables, usePrompts } from "@/hooks/use-prompts";
import type { Prompt } from "@/lib/api/prompts";
import { errorMessage } from "@/lib/errors";
import { stagger } from "@/lib/motion";
import { CopyPromptDialog } from "./copy-prompt-dialog";
import { filterPrompts } from "./filter";
import { PromptCard } from "./prompt-card";
import { PromptDialog } from "./prompt-dialog";

const ALL_PROJECTS = { search: null, tag: null, include_archived: true };

/** All prompts, or opened on one project's prompts when `projectId` is given. */
export function PromptLibrary({ projectId = null }: { projectId?: string | null }) {
  const prompts = usePrompts();
  const projects = useProjects(ALL_PROJECTS);
  const variables = usePromptVariables();
  const copy = useCopyPrompt();
  const [search, setSearch] = useState("");
  const [scope, setScope] = useState(projectId ?? "all");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [editing, setEditing] = useState<Prompt | null>(null);
  const [creating, setCreating] = useState(false);
  const [copying, setCopying] = useState<{ prompt: Prompt; variables: string[] } | null>(null);

  const projectList = projects.data ?? [];
  const names = new Map(projectList.map((project) => [project.id, project.name]));
  const searching = search.trim() !== "" || favoritesOnly;
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
          className="w-48"
          value={scope}
          options={[
            { value: "all", label: "All prompts" },
            { value: "global", label: "Global" },
            ...projectList.map((project) => ({ value: project.id, label: project.name })),
          ]}
          onChange={setScope}
        />
        <Chip selected={favoritesOnly} onClick={() => setFavoritesOnly(!favoritesOnly)}>
          Favorites
        </Chip>
        {copy.isSuccess && <span className="text-sm text-muted-foreground">Copied</span>}
        <Button className="ml-auto" onClick={() => setCreating(true)}>
          New prompt
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}

      {prompts.isSuccess && visible.length === 0 && (
        <EmptyState
          icon={MessageSquareText}
          area="workflow"
          line={
            searching
              ? "No prompts match the filters."
              : prompts.data.length === 0
                ? "No prompts yet. Save the ones you reuse."
                : "No prompts here yet."
          }
          action={
            searching ? (
              <Button
                variant="outline"
                onClick={() => {
                  setSearch("");
                  setFavoritesOnly(false);
                }}
              >
                Clear filters
              </Button>
            ) : (
              <Button onClick={() => setCreating(true)}>New prompt</Button>
            )
          }
        />
      )}
      <div className="grid gap-3 lg:grid-cols-2">
        {visible.map((prompt, index) => (
          <div key={prompt.id} className="stagger-in" style={stagger(index)}>
            <PromptCard
              prompt={prompt}
              projectName={prompt.project_id ? (names.get(prompt.project_id) ?? "Project") : null}
              onCopy={startCopy}
              onEdit={setEditing}
            />
          </div>
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
