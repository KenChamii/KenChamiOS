import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { useCreateProject, useUpdateProject } from "@/hooks/use-projects";
import type { ProjectInput, ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";
import { ProjectForm } from "./project-form";

type DialogProps = { open: boolean; onOpenChange: (open: boolean) => void };

export const EMPTY_PROJECT: ProjectInput = {
  name: "",
  description: "",
  repo_path: null,
  color: null,
  tags: [],
  archived: false,
};

export function NewProjectDialog({
  open,
  onOpenChange,
  onCreated,
}: DialogProps & { onCreated: (project: ProjectSummary) => void }) {
  const create = useCreateProject();
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent aria-describedby={undefined}>
        <DialogTitle className="mb-4 text-lg font-semibold">New project</DialogTitle>
        <ProjectForm
          initial={EMPTY_PROJECT}
          submitLabel="Create project"
          pending={create.isPending}
          error={create.error ? errorMessage(create.error) : null}
          onSubmit={(input) => create.mutate(input, { onSuccess: onCreated })}
        />
      </DialogContent>
    </Dialog>
  );
}

export function EditProjectDialog({
  project,
  open,
  onOpenChange,
}: DialogProps & { project: ProjectSummary }) {
  const update = useUpdateProject(project.id);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent aria-describedby={undefined}>
        <DialogTitle className="mb-4 text-lg font-semibold">Edit project</DialogTitle>
        <ProjectForm
          initial={project}
          submitLabel="Save changes"
          pending={update.isPending}
          error={update.error ? errorMessage(update.error) : null}
          onSubmit={(input) => update.mutate(input, { onSuccess: () => onOpenChange(false) })}
        />
      </DialogContent>
    </Dialog>
  );
}
