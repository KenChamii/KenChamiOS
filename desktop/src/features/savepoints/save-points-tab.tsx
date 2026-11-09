import { save } from "@tauri-apps/plugin-dialog";
import { Flag } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { useExportSavePoints, useSavePoints } from "@/hooks/use-save-points";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";
import { SavePointDialog } from "./save-point-dialog";
import { SavePointEntry } from "./save-point-entry";
import { SessionNoteDialog } from "./session-note-dialog";

export function SavePointsTab({ project }: { project: ProjectSummary }) {
  const savePoints = useSavePoints(project.id);
  const exportMarkdown = useExportSavePoints(project.id);
  const [creating, setCreating] = useState(false);
  const [pastingNote, setPastingNote] = useState(false);

  async function handleExport() {
    const path = await save({
      defaultPath: `${project.slug}-save-points.md`,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (path) exportMarkdown.mutate(path);
  }

  const error = savePoints.error ?? exportMarkdown.error;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2">
        <Button onClick={() => setCreating(true)}>Create save point</Button>
        <Button variant="outline" onClick={() => setPastingNote(true)}>
          Paste session note
        </Button>
        <Button variant="outline" disabled={exportMarkdown.isPending} onClick={handleExport}>
          Export to Markdown
        </Button>
        {exportMarkdown.isSuccess && (
          <span className="text-sm text-muted-foreground">Exported</span>
        )}
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {savePoints.data?.length === 0 && (
        <EmptyState
          icon={Flag}
          area="projects"
          line="No save points yet. Make one when you stop, so you can pick up later."
          action={<Button onClick={() => setCreating(true)}>Create save point</Button>}
        />
      )}
      <ol className="flex flex-col gap-2">
        {savePoints.data?.map((savePoint, index) => (
          <SavePointEntry key={savePoint.id} savePoint={savePoint} initiallyOpen={index === 0} />
        ))}
      </ol>
      <SavePointDialog project={project} open={creating} onOpenChange={setCreating} />
      <SessionNoteDialog project={project} open={pastingNote} onOpenChange={setPastingNote} />
    </div>
  );
}
