import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTitle } from "@/components/ui/sheet";
import { useDeleteMedia } from "@/hooks/use-media";
import type { LibraryEntry } from "@/lib/api/media";
import { errorMessage } from "@/lib/errors";
import { Cover } from "./cover";
import { EntryForm } from "./entry-form";
import { TYPE_LABELS } from "./meta";
import { ProgressEditor } from "./progress-editor";

type EntrySheetProps = { entry: LibraryEntry | null; onClose: () => void };

export function EntrySheet({ entry, onClose }: EntrySheetProps) {
  return (
    <Sheet open={entry !== null} onOpenChange={(open) => !open && onClose()}>
      <SheetContent aria-describedby={undefined}>
        <SheetTitle className="mb-4 text-lg font-semibold">{entry?.title}</SheetTitle>
        {entry && <EntryDetail entry={entry} onClose={onClose} />}
      </SheetContent>
    </Sheet>
  );
}

function EntryDetail({ entry, onClose }: { entry: LibraryEntry; onClose: () => void }) {
  const remove = useDeleteMedia();
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const facts = [
    TYPE_LABELS[entry.media_type],
    entry.year !== null ? String(entry.year) : null,
    ...entry.genres,
  ].filter(Boolean);

  return (
    <div className="flex flex-col gap-6">
      <div className="flex gap-4">
        <Cover
          title={entry.title}
          path={entry.cover_path}
          url={entry.cover_url}
          mediaType={entry.media_type}
          className="w-28 shrink-0"
        />
        <div className="flex min-w-0 flex-col gap-2">
          <p className="text-sm text-muted-foreground">{facts.join(" · ")}</p>
          {entry.synopsis && (
            <p className="line-clamp-6 whitespace-pre-line text-sm">{entry.synopsis}</p>
          )}
        </div>
      </div>
      <ProgressEditor key={`${entry.id}:${entry.progress}`} entry={entry} />
      <EntryForm key={`${entry.id}:${entry.updated_at}`} entry={entry} />
      {remove.error && <p className="text-sm text-destructive">{errorMessage(remove.error)}</p>}
      <div className="flex gap-2">
        {confirmingDelete ? (
          <>
            <Button
              variant="destructive"
              disabled={remove.isPending}
              onClick={() => remove.mutate(entry.id, { onSuccess: onClose })}
            >
              Confirm remove
            </Button>
            <Button variant="ghost" onClick={() => setConfirmingDelete(false)}>
              Cancel
            </Button>
          </>
        ) : (
          <Button variant="ghost" onClick={() => setConfirmingDelete(true)}>
            Remove from library
          </Button>
        )}
      </div>
    </div>
  );
}
