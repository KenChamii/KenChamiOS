import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useUpdateMedia } from "@/hooks/use-media";
import type { LibraryEntry } from "@/lib/api/media";
import { errorMessage } from "@/lib/errors";
import { draftOf, inputOf } from "./entry-input";
import { STATUS_LABELS, STATUSES, TOTAL_LABELS } from "./meta";

export function EntryForm({ entry }: { entry: LibraryEntry }) {
  const update = useUpdateMedia();
  const [draft, setDraft] = useState(() => draftOf(entry));
  const input = inputOf(draft);
  const isGame = entry.media_type === "game";
  const totalLabel = TOTAL_LABELS[entry.media_type];

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (input) update.mutate({ id: entry.id, input });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <Field label="Status">
          <Select
            value={draft.status}
            onChange={(e) => setDraft({ ...draft, status: e.target.value as typeof draft.status })}
          >
            {STATUSES.map((status) => (
              <option key={status} value={status}>
                {STATUS_LABELS[entry.media_type][status]}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Rating (1-10)">
          <Input
            type="number"
            min={1}
            max={10}
            value={draft.rating}
            onChange={(e) => setDraft({ ...draft, rating: e.target.value })}
          />
        </Field>
        {totalLabel && (
          <Field label={totalLabel}>
            <Input
              type="number"
              min={0}
              placeholder="Unknown"
              value={draft.total}
              onChange={(e) => setDraft({ ...draft, total: e.target.value })}
            />
          </Field>
        )}
        {isGame && (
          <Field label="Platform">
            <Input
              placeholder="PC, Switch..."
              value={draft.platform}
              onChange={(e) => setDraft({ ...draft, platform: e.target.value })}
            />
          </Field>
        )}
        <Field label="Started">
          <Input
            type="date"
            value={draft.startedAt}
            onChange={(e) => setDraft({ ...draft, startedAt: e.target.value })}
          />
        </Field>
        <Field label="Finished">
          <Input
            type="date"
            value={draft.finishedAt}
            onChange={(e) => setDraft({ ...draft, finishedAt: e.target.value })}
          />
        </Field>
      </div>
      <Field label="Tags (comma separated)">
        <Input value={draft.tags} onChange={(e) => setDraft({ ...draft, tags: e.target.value })} />
      </Field>
      <Field label="Notes">
        <Textarea
          value={draft.notes}
          onChange={(e) => setDraft({ ...draft, notes: e.target.value })}
        />
      </Field>
      {update.error && <p className="text-sm text-destructive">{errorMessage(update.error)}</p>}
      <div className="flex items-center gap-3">
        <Button type="submit" disabled={!input || update.isPending}>
          Save
        </Button>
        {!input && (
          <span className="text-sm text-muted-foreground">Rating is 1-10, total is a number.</span>
        )}
        {update.isSuccess && <span className="text-sm text-muted-foreground">Saved</span>}
      </div>
    </form>
  );
}
