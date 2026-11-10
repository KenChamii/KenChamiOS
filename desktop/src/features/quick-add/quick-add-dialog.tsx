import { useMatch } from "react-router-dom";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Tabs } from "@/components/ui/tabs";
import { useProjects } from "@/hooks/use-projects";
import { useOverlays, type QuickAddMode } from "@/stores/overlays";
import { QuickIdeaForm, QuickMediaForm, QuickPromptForm } from "./other-forms";
import { QuickTaskForm } from "./task-form";

const MODES: { id: QuickAddMode; label: string }[] = [
  { id: "task", label: "Task" },
  { id: "idea", label: "Idea" },
  { id: "prompt", label: "Prompt" },
  { id: "media", label: "Media" },
];

const ACTIVE_PROJECTS = { search: null, tag: null, include_archived: false };

export function QuickAddDialog() {
  const { quickAdd, quickAddText, openQuickAdd, closeQuickAdd } = useOverlays();
  const projects = useProjects(ACTIVE_PROJECTS);
  // inside a project, new tasks and prompts go there by default
  const projectId = useMatch("/projects/:id/*")?.params.id ?? null;
  const list = projects.data ?? [];
  const current = list.some((project) => project.id === projectId) ? projectId : null;

  return (
    <Dialog open={quickAdd !== null} onOpenChange={(open) => !open && closeQuickAdd()}>
      <DialogContent aria-describedby={undefined} className="flex max-w-lg flex-col gap-5">
        <DialogTitle className="text-lg font-semibold">Quick add</DialogTitle>
        {quickAdd && (
          <>
            <Tabs tabs={MODES} value={quickAdd} onChange={openQuickAdd} />
            {quickAdd === "task" && projects.data && (
              <QuickTaskForm
                projects={list}
                defaultProjectId={current}
                initialText={quickAddText}
                onDone={closeQuickAdd}
              />
            )}
            {quickAdd === "idea" && <QuickIdeaForm onDone={closeQuickAdd} />}
            {quickAdd === "prompt" && (
              <QuickPromptForm projects={list} defaultProjectId={current} onDone={closeQuickAdd} />
            )}
            {quickAdd === "media" && <QuickMediaForm onDone={closeQuickAdd} />}
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
