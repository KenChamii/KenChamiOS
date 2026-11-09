import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { useDeleteProject, useSetStage, useUpdateProject } from "@/hooks/use-projects";
import { updateProject, type ProjectSummary } from "@/lib/api/projects";
import { GitCard } from "@/features/savepoints/git-card";
import { ResumePanel } from "@/features/savepoints/resume-panel";
import { errorMessage } from "@/lib/errors";
import { useToasts } from "@/stores/toasts";
import { EditProjectDialog } from "./project-dialog";
import { StagePipeline } from "./stage-pipeline";

export function OverviewTab({ project }: { project: ProjectSummary }) {
  const navigate = useNavigate();
  const [editing, setEditing] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const setStage = useSetStage(project.id);
  const update = useUpdateProject(project.id);
  const remove = useDeleteProject(project.id);

  const error = setStage.error ?? update.error ?? remove.error;

  function toggleArchived() {
    const archived = !project.archived;
    update.mutate(
      { ...project, archived },
      {
        onSuccess: () => {
          if (!archived) return;
          useToasts
            .getState()
            .show("Project archived", () =>
              updateProject(project.id, { ...project, archived: false }),
            );
        },
      },
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <Card className="flex flex-col gap-3">
        <h2 className="font-semibold tracking-tight">Stage</h2>
        <StagePipeline
          stage={project.stage}
          disabled={setStage.isPending}
          onChange={(stage) => setStage.mutate(stage)}
        />
      </Card>

      <ResumePanel project={project} />

      <GitCard projectId={project.id} />

      <Card className="flex flex-col gap-3">
        <h2 className="font-semibold tracking-tight">Details</h2>
        <p className="text-sm">
          {project.description || <span className="text-muted-foreground">No description.</span>}
        </p>
        <dl className="grid grid-cols-[8rem_1fr] gap-2 text-sm">
          <dt className="text-muted-foreground">Repository</dt>
          <dd className="break-all">{project.repo_path ?? "Not set"}</dd>
          <dt className="text-muted-foreground">Tags</dt>
          <dd className="flex flex-wrap gap-1">
            {project.tags.length > 0
              ? project.tags.map((tag) => <Badge key={tag}>{tag}</Badge>)
              : "None"}
          </dd>
          <dt className="text-muted-foreground">Progress</dt>
          <dd className="flex flex-col gap-1">
            <Progress value={project.tasks_done} max={project.tasks_total} area="projects" />
            <span className="text-xs text-muted-foreground">
              {project.tasks_done} of {project.tasks_total} tasks done
            </span>
          </dd>
        </dl>
      </Card>

      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}

      <div className="flex gap-2">
        <Button variant="outline" onClick={() => setEditing(true)}>
          Edit
        </Button>
        <Button variant="outline" disabled={update.isPending} onClick={toggleArchived}>
          {project.archived ? "Unarchive" : "Archive"}
        </Button>
        {confirmingDelete ? (
          <>
            <Button
              variant="destructive"
              disabled={remove.isPending}
              onClick={() => remove.mutate(undefined, { onSuccess: () => navigate("/projects") })}
            >
              Confirm delete
            </Button>
            <Button variant="ghost" onClick={() => setConfirmingDelete(false)}>
              Cancel
            </Button>
          </>
        ) : (
          <Button variant="ghost" onClick={() => setConfirmingDelete(true)}>
            Delete
          </Button>
        )}
      </div>

      <EditProjectDialog project={project} open={editing} onOpenChange={setEditing} />
    </div>
  );
}
