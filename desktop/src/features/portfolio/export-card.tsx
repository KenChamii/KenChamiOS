import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useCopyPortfolio, usePreviewPortfolio } from "@/hooks/use-portfolio";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";
import { ExportDialog } from "./export-dialog";

export function ExportCard({ project }: { project: ProjectSummary }) {
  const preview = usePreviewPortfolio(project.id);
  const copy = useCopyPortfolio(project.id);
  const [dialogOpen, setDialogOpen] = useState(false);
  const error = preview.error ?? copy.error;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">Portfolio page</h2>
      <p className="text-sm text-muted-foreground">
        {project.stage === "done"
          ? "The project is done. Export the write-up with its screenshots."
          : "The project is not done yet, but the page can be exported at any time."}
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant={project.stage === "done" ? "primary" : "outline"}
          disabled={preview.isPending}
          onClick={() => preview.mutate(undefined, { onSuccess: () => setDialogOpen(true) })}
        >
          Preview and export
        </Button>
        <Button variant="outline" disabled={copy.isPending} onClick={() => copy.mutate()}>
          Copy Markdown
        </Button>
        {copy.isSuccess && <span className="text-sm text-muted-foreground">Copied</span>}
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <ExportDialog
        projectId={project.id}
        markdown={dialogOpen ? (preview.data ?? null) : null}
        onClose={() => setDialogOpen(false)}
      />
    </Card>
  );
}
