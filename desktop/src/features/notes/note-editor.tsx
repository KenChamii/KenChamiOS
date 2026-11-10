import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { MarkdownEditor } from "@/features/docs/markdown-editor";
import { MarkdownPreview } from "@/features/docs/markdown-preview";
import { useDailyNote, useSaveDailyNote } from "@/hooks/use-daily-notes";
import { saveDailyNote } from "@/lib/api/daily-notes";
import { cn } from "@/lib/cn";
import type { Day } from "@/lib/days";
import { errorMessage } from "@/lib/errors";

const AUTOSAVE_MS = 800;

/** The note of one day; mount it with `key={day}` so each day starts fresh. */
export function NoteEditor({ day }: { day: Day }) {
  const note = useDailyNote(day);
  if (note.isPending) return <p className="text-sm text-muted-foreground">Loading...</p>;
  if (note.error) return <p className="text-sm text-destructive">{errorMessage(note.error)}</p>;
  return <AutosaveEditor day={day} initialBody={note.data?.body ?? ""} />;
}

function AutosaveEditor({ day, initialBody }: { day: Day; initialBody: string }) {
  const save = useSaveDailyNote();
  const queryClient = useQueryClient();
  const [draft, setDraft] = useState(initialBody);
  const [savedBody, setSavedBody] = useState(initialBody);
  const [previewing, setPreviewing] = useState(false);
  const unsaved = useRef<string | null>(null);
  const dirty = draft !== savedBody;
  const { mutate } = save;

  useEffect(() => {
    unsaved.current = dirty ? draft : null;
    if (!dirty) return;
    const timer = setTimeout(
      () => mutate({ day, body: draft }, { onSuccess: (saved) => setSavedBody(saved.body) }),
      AUTOSAVE_MS,
    );
    return () => clearTimeout(timer);
  }, [day, draft, dirty, mutate]);

  // switching days or pages must not lose the last keystrokes
  useEffect(
    () => () => {
      if (unsaved.current === null) return;
      void saveDailyNote(day, unsaved.current).then(() =>
        queryClient.invalidateQueries({ queryKey: ["daily-notes"] }),
      );
    },
    [day, queryClient],
  );

  const status = save.error
    ? errorMessage(save.error)
    : dirty || save.isPending
      ? "Saving..."
      : "Saved";

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <Button variant={previewing ? "outline" : "primary"} onClick={() => setPreviewing(false)}>
          Edit
        </Button>
        <Button variant={previewing ? "primary" : "outline"} onClick={() => setPreviewing(true)}>
          Preview
        </Button>
        <span
          className={cn(
            "ml-auto text-sm",
            save.error ? "text-destructive" : "text-muted-foreground",
          )}
        >
          {status}
        </span>
      </div>
      {previewing ? (
        <MarkdownPreview source={draft} />
      ) : (
        <MarkdownEditor initialValue={draft} onChange={setDraft} />
      )}
    </div>
  );
}
