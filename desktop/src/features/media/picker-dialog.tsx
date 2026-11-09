import { Dices } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import type { LibraryEntry, MediaType } from "@/lib/api/media";
import { Cover } from "./cover";
import { distinctValues } from "./library-filter";
import { TYPES, TYPE_LABELS } from "./meta";
import { backlog, NO_PICKER_FILTER, pickNext, remaining, type PickerFilter } from "./picker";

type PickerDialogProps = {
  entries: LibraryEntry[];
  open: boolean;
  onClose: () => void;
  onOpenEntry: (id: string) => void;
};

export function PickerDialog({ entries, open, onClose, onOpenEntry }: PickerDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent aria-describedby={undefined} className="max-h-[85vh] overflow-auto">
        <DialogTitle className="mb-4 text-lg font-semibold">What next?</DialogTitle>
        {open && (
          <PickerBody
            entries={entries}
            onOpenEntry={(id) => {
              onClose();
              onOpenEntry(id);
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}

function PickerBody({
  entries,
  onOpenEntry,
}: {
  entries: LibraryEntry[];
  onOpenEntry: (id: string) => void;
}) {
  const [filter, setFilter] = useState<PickerFilter>(NO_PICKER_FILTER);
  const [limit, setLimit] = useState("");
  const [picked, setPicked] = useState<LibraryEntry | null>(null);
  const candidates = backlog(entries, filter);
  const left = picked ? remaining(picked) : null;

  function changeFilter(next: PickerFilter) {
    setFilter(next);
    setPicked(null);
  }

  function changeLimit(text: string) {
    setLimit(text);
    const value = Number(text);
    const valid = text.trim() !== "" && Number.isFinite(value) && value >= 0;
    changeFilter({ ...filter, maxLeft: valid ? Math.floor(value) : null });
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-2 gap-3">
        <Field label="Type">
          <Select
            value={filter.type}
            onChange={(e) => changeFilter({ ...filter, type: e.target.value as MediaType | "all" })}
          >
            <option value="all">Anything</option>
            {TYPES.map((type) => (
              <option key={type.id} value={type.id}>
                {type.label}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Genre">
          <Select
            value={filter.genre}
            onChange={(e) => changeFilter({ ...filter, genre: e.target.value })}
          >
            <option value="">Any genre</option>
            {distinctValues(entries, "genres").map((genre) => (
              <option key={genre}>{genre}</option>
            ))}
          </Select>
        </Field>
        <Field label="Mood tag">
          <Select
            value={filter.tag}
            onChange={(e) => changeFilter({ ...filter, tag: e.target.value })}
          >
            <option value="">Any mood</option>
            {distinctValues(entries, "tags").map((tag) => (
              <option key={tag}>{tag}</option>
            ))}
          </Select>
        </Field>
        <Field label="At most this many left">
          <Input
            type="number"
            min={0}
            placeholder="No limit"
            value={limit}
            onChange={(e) => changeLimit(e.target.value)}
          />
        </Field>
      </div>

      <p className="text-sm text-muted-foreground">
        {candidates.length} planned {candidates.length === 1 ? "title fits" : "titles fit"}.
      </p>

      {picked && (
        <button
          type="button"
          onClick={() => onOpenEntry(picked.id)}
          className="flex items-center gap-3 rounded-md border border-border p-3 text-left"
        >
          <Cover
            title={picked.title}
            path={picked.cover_path}
            url={picked.cover_url}
            className="w-16 shrink-0"
          />
          <span className="flex flex-col gap-1">
            <span className="font-medium">{picked.title}</span>
            <span className="text-xs text-muted-foreground">
              {TYPE_LABELS[picked.media_type]}
              {picked.year !== null && ` · ${picked.year}`}
              {left !== null && ` · ${left} left`}
            </span>
          </span>
        </button>
      )}

      <Button
        disabled={candidates.length === 0}
        onClick={() => setPicked(pickNext(candidates, picked?.id ?? null))}
      >
        <Dices size={16} />
        {picked ? "Pick another" : "Pick for me"}
      </Button>
    </div>
  );
}
