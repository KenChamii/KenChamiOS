import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { StageBadge } from "@/features/projects/stage-badge";
import { useGitInfo } from "@/hooks/use-git";
import { useCreateSavePoint } from "@/hooks/use-save-points";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";
import { branchLabel, commitLabel } from "./meta";

type SavePointDialogProps = {
  project: ProjectSummary;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function SavePointDialog({ project, open, onOpenChange }: SavePointDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent aria-describedby={undefined} className="max-w-xl">
        <DialogTitle className="mb-4 text-lg font-semibold">Create save point</DialogTitle>
        <SavePointForm project={project} onDone={() => onOpenChange(false)} />
      </DialogContent>
    </Dialog>
  );
}

function SavePointForm({ project, onDone }: { project: ProjectSummary; onDone: () => void }) {
  const create = useCreateSavePoint(project.id);
  const git = useGitInfo(project.id);
  const [note, setNote] = useState("");
  const [doneMd, setDoneMd] = useState("");
  const [nextMd, setNextMd] = useState("");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    create.mutate({ note, done_md: doneMd, next_md: nextMd }, { onSuccess: onDone });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
        <StageBadge stage={project.stage} />
        {git.data?.state === "repo" && (
          <span>{commitLabel(git.data.branch, git.data.head) ?? branchLabel(git.data)}</span>
        )}
      </div>
      <Field label="Note">
        <Input value={note} onChange={(e) => setNote(e.target.value)} autoFocus required />
      </Field>
      <Field label="Done (markdown)">
        <Textarea value={doneMd} onChange={(e) => setDoneMd(e.target.value)} />
      </Field>
      <Field label="Next (markdown)">
        <Textarea value={nextMd} onChange={(e) => setNextMd(e.target.value)} />
      </Field>
      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}
      <Button type="submit" disabled={create.isPending || !note.trim()}>
        Create save point
      </Button>
    </form>
  );
}
