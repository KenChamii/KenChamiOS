import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import { usePreviewSessionNote } from "@/hooks/use-session-notes";
import type { ProjectSummary } from "@/lib/api/projects";
import type { NotePreview } from "@/lib/api/session-notes";
import { errorMessage } from "@/lib/errors";
import { SessionNotePreview } from "./session-note-preview";

type SessionNoteDialogProps = {
  project: ProjectSummary;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function SessionNoteDialog({ project, open, onOpenChange }: SessionNoteDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        aria-describedby={undefined}
        className="flex max-h-[90vh] max-w-2xl flex-col overflow-y-auto"
      >
        <DialogTitle className="mb-4 text-lg font-semibold">Session note</DialogTitle>
        <SessionNoteSteps project={project} onDone={() => onOpenChange(false)} />
      </DialogContent>
    </Dialog>
  );
}

function SessionNoteSteps({ project, onDone }: { project: ProjectSummary; onDone: () => void }) {
  const parse = usePreviewSessionNote(project.id);
  const [text, setText] = useState("");
  const [preview, setPreview] = useState<NotePreview | null>(null);

  if (preview) {
    return (
      <SessionNotePreview
        project={project}
        text={text}
        preview={preview}
        onBack={() => setPreview(null)}
        onDone={onDone}
      />
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-muted-foreground">
        Paste the summary your coding session ended with. Sections named Done, Next and Changed
        files, and checkbox lists, are picked out; the rest becomes the note.
      </p>
      <Textarea
        value={text}
        onChange={(event) => setText(event.target.value)}
        className="min-h-64 font-mono"
        aria-label="Session summary"
        autoFocus
      />
      {parse.error && <p className="text-sm text-destructive">{errorMessage(parse.error)}</p>}
      <Button
        disabled={parse.isPending || !text.trim()}
        onClick={() => parse.mutate(text, { onSuccess: setPreview })}
      >
        Preview
      </Button>
    </div>
  );
}
