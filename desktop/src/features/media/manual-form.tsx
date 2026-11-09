import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useAddManualMedia } from "@/hooks/use-media";
import type { MediaStatus, MediaType } from "@/lib/api/media";
import { errorMessage } from "@/lib/errors";
import { parseTags } from "@/lib/tags";
import { manualInputOf } from "./manual-input";
import { STATUS_LABELS, STATUSES, TOTAL_LABELS, TYPE_LABELS } from "./meta";

type ManualFormProps = { initialType: MediaType; onBack: () => void; onDone: () => void };

export function ManualForm({ initialType, onBack, onDone }: ManualFormProps) {
  const add = useAddManualMedia();
  const [type, setType] = useState(initialType);
  const [status, setStatus] = useState<MediaStatus>("planned");
  const [title, setTitle] = useState("");
  const [year, setYear] = useState("");
  const [total, setTotal] = useState("");
  const [genres, setGenres] = useState("");
  const input = manualInputOf({ type, status, title, year, total, genres: parseTags(genres) });

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (input) add.mutate(input, { onSuccess: onDone });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      <p className="text-sm text-muted-foreground">
        Works offline. Nothing is looked up, so fill in what you know.
      </p>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Type">
          <Select value={type} onChange={(e) => setType(e.target.value as MediaType)}>
            {(Object.keys(TYPE_LABELS) as MediaType[]).map((value) => (
              <option key={value} value={value}>
                {TYPE_LABELS[value]}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Status">
          <Select value={status} onChange={(e) => setStatus(e.target.value as MediaStatus)}>
            {STATUSES.map((value) => (
              <option key={value} value={value}>
                {STATUS_LABELS[type][value]}
              </option>
            ))}
          </Select>
        </Field>
      </div>
      <Field label="Title">
        <Input required value={title} onChange={(e) => setTitle(e.target.value)} />
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Year">
          <Input type="number" value={year} onChange={(e) => setYear(e.target.value)} />
        </Field>
        {TOTAL_LABELS[type] && (
          <Field label={TOTAL_LABELS[type]}>
            <Input
              type="number"
              min={0}
              placeholder="Unknown"
              value={total}
              onChange={(e) => setTotal(e.target.value)}
            />
          </Field>
        )}
      </div>
      <Field label="Genres (comma separated)">
        <Input value={genres} onChange={(e) => setGenres(e.target.value)} />
      </Field>
      {add.error && <p className="text-sm text-destructive">{errorMessage(add.error)}</p>}
      <div className="flex justify-end gap-2">
        <Button type="button" variant="ghost" onClick={onBack}>
          Back to search
        </Button>
        <Button type="submit" disabled={!input || add.isPending}>
          Add
        </Button>
      </div>
    </form>
  );
}
