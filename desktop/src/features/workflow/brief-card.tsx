import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Select } from "@/components/ui/select";
import {
  useBriefTarget,
  useBriefTargets,
  useCopyBrief,
  usePreviewBrief,
  useSetBriefTarget,
} from "@/hooks/use-brief";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";
import { BriefExportDialog } from "./brief-export-dialog";

export function BriefCard({ project }: { project: ProjectSummary }) {
  const targets = useBriefTargets();
  const target = useBriefTarget(project.id);
  const setTarget = useSetBriefTarget(project.id);
  const copy = useCopyBrief(project.id);
  const preview = usePreviewBrief(project.id);
  const [dialogOpen, setDialogOpen] = useState(false);

  const error = targets.error ?? target.error ?? setTarget.error ?? copy.error ?? preview.error;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Project brief</h2>
      <p className="text-sm text-muted-foreground">
        A summary of the project for a coding assistant: plan, open tasks and the last save point.
      </p>
      <Field label="Target file">
        <Select
          value={target.data ?? ""}
          disabled={setTarget.isPending}
          onChange={(event) => setTarget.mutate(event.target.value)}
        >
          {targets.data?.map((file) => (
            <option key={file} value={file}>
              {file}
            </option>
          ))}
        </Select>
      </Field>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant="outline"
          disabled={preview.isPending || !project.repo_path}
          onClick={() => preview.mutate(undefined, { onSuccess: () => setDialogOpen(true) })}
        >
          Export to file
        </Button>
        <Button variant="outline" disabled={copy.isPending} onClick={() => copy.mutate()}>
          Copy to clipboard
        </Button>
        {copy.isSuccess && <span className="text-sm text-muted-foreground">Copied</span>}
        {!project.repo_path && (
          <span className="text-sm text-muted-foreground">Set a repository folder to export.</span>
        )}
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <BriefExportDialog
        projectId={project.id}
        preview={dialogOpen ? (preview.data ?? null) : null}
        onClose={() => setDialogOpen(false)}
      />
    </Card>
  );
}
