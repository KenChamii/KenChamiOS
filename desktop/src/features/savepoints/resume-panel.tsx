import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useSavePoints } from "@/hooks/use-save-points";
import type { ProjectSummary } from "@/lib/api/projects";
import { timeAgo } from "@/lib/dates";
import { commitLabel } from "./meta";
import { SavePointBody } from "./save-point-body";
import { SavePointDialog } from "./save-point-dialog";
import { SessionNoteDialog } from "./session-note-dialog";

export function ResumePanel({ project }: { project: ProjectSummary }) {
  const savePoints = useSavePoints(project.id);
  const [creating, setCreating] = useState(false);
  const [pastingNote, setPastingNote] = useState(false);
  const last = savePoints.data?.[0];
  const git = last && commitLabel(last.git_branch, last.git_commit);

  return (
    <Card className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-2">
        <h2 className="font-semibold tracking-tight">Resume</h2>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => setPastingNote(true)}>
            Paste session note
          </Button>
          <Button variant="outline" size="sm" onClick={() => setCreating(true)}>
            Create save point
          </Button>
        </div>
      </div>
      {last ? (
        <>
          <p className="text-sm text-muted-foreground">
            Last save point {timeAgo(last.created_at)}
            {git && `, ${git}`}
          </p>
          <p className="text-sm font-medium">{last.note}</p>
          {last.next_md.trim() ? (
            <SavePointBody title="Next" source={last.next_md} />
          ) : (
            <p className="text-sm text-muted-foreground">No next steps were written down.</p>
          )}
        </>
      ) : (
        <p className="text-sm text-muted-foreground">
          {savePoints.isPending ? "Loading..." : "No save points yet."}
        </p>
      )}
      <SavePointDialog project={project} open={creating} onOpenChange={setCreating} />
      <SessionNoteDialog project={project} open={pastingNote} onOpenChange={setPastingNote} />
    </Card>
  );
}
