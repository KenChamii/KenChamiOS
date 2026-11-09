import { openUrl } from "@tauri-apps/plugin-opener";
import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Tabs } from "@/components/ui/tabs";
import { useAddMedia, useMediaSearch } from "@/hooks/use-media";
import type { LibraryEntry, MediaStatus, MediaType } from "@/lib/api/media";
import { errorMessage } from "@/lib/errors";
import { Cover } from "./cover";
import { ManualForm } from "./manual-form";
import { KEYED_TYPES, SOURCE_LINKS, STATUS_LABELS, STATUSES, TYPES } from "./meta";

type SearchDialogProps = { entries: LibraryEntry[]; open: boolean; onClose: () => void };

export function SearchDialog({ open, onClose, entries }: SearchDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent aria-describedby={undefined} className="max-h-[85vh] max-w-2xl overflow-auto">
        <DialogTitle className="mb-4 text-lg font-semibold">Add to library</DialogTitle>
        {open && <SearchBody entries={entries} onClose={onClose} />}
      </DialogContent>
    </Dialog>
  );
}

function SearchBody({ entries, onClose }: { entries: LibraryEntry[]; onClose: () => void }) {
  const [type, setType] = useState<MediaType>("anime");
  const [text, setText] = useState("");
  const [submitted, setSubmitted] = useState("");
  const [status, setStatus] = useState<MediaStatus>("planned");
  const [manual, setManual] = useState(false);

  if (manual)
    return <ManualForm initialType={type} onBack={() => setManual(false)} onDone={onClose} />;

  return (
    <div className="flex flex-col gap-4">
      <Tabs tabs={TYPES} value={type} onChange={setType} />
      <SearchForm
        text={text}
        onText={setText}
        onSubmit={() => setSubmitted(text.trim())}
        status={status}
        onStatus={setStatus}
        type={type}
      />
      <Results
        key={type}
        query={submitted}
        type={type}
        status={status}
        entries={entries}
        onClose={onClose}
      />
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <button type="button" className="underline" onClick={() => openUrl(SOURCE_LINKS[type].url)}>
          Data from {SOURCE_LINKS[type].name}
        </button>
        <Button variant="outline" size="sm" onClick={() => setManual(true)}>
          Can&apos;t find it? Add manually
        </Button>
      </div>
    </div>
  );
}

type SearchFormProps = {
  text: string;
  onText: (text: string) => void;
  onSubmit: () => void;
  status: MediaStatus;
  onStatus: (status: MediaStatus) => void;
  type: MediaType;
};

function SearchForm({ text, onText, onSubmit, status, onStatus, type }: SearchFormProps) {
  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    onSubmit();
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-wrap items-center gap-2">
      <Input
        className="min-w-48 flex-1"
        autoFocus
        placeholder="Search a title"
        aria-label="Search a title"
        value={text}
        onChange={(event) => onText(event.target.value)}
      />
      <Button type="submit" disabled={text.trim() === ""}>
        Search
      </Button>
      <Select
        aria-label="Add as"
        value={status}
        onChange={(event) => onStatus(event.target.value as MediaStatus)}
      >
        {STATUSES.map((value) => (
          <option key={value} value={value}>
            {STATUS_LABELS[type][value]}
          </option>
        ))}
      </Select>
    </form>
  );
}

type ResultsProps = {
  query: string;
  type: MediaType;
  status: MediaStatus;
  entries: LibraryEntry[];
  onClose: () => void;
};

function Results({ query, type, status, entries, onClose }: ResultsProps) {
  const search = useMediaSearch(query, type);
  const add = useAddMedia();
  const owned = new Set(
    entries.filter((e) => e.media_type === type && e.external_id).map((e) => e.external_id),
  );

  if (query === "") return null;
  if (search.isPending) return <p className="text-sm text-muted-foreground">Searching...</p>;
  if (search.error) {
    return (
      <p className="text-sm text-destructive">
        {errorMessage(search.error)}.{" "}
        {KEYED_TYPES.includes(type) && (
          <Link to="/settings" onClick={onClose} className="underline">
            Open Settings
          </Link>
        )}
      </p>
    );
  }
  if (search.data.length === 0) {
    return <p className="text-sm text-muted-foreground">Nothing found. Try another title.</p>;
  }

  return (
    <>
      <ul className="flex flex-col divide-y divide-border">
        {search.data.map((hit) => (
          <li key={hit.external_id} className="flex gap-3 py-3">
            <Cover title={hit.title} path={null} url={hit.cover_url} className="w-14 shrink-0" />
            <div className="flex min-w-0 flex-1 flex-col gap-1">
              <span className="text-sm font-medium">
                {hit.title}
                {hit.year !== null && ` (${hit.year})`}
              </span>
              <span className="text-xs text-muted-foreground">{hit.genres.join(", ")}</span>
              <span className="line-clamp-2 text-xs">{hit.synopsis}</span>
            </div>
            <Button
              size="sm"
              disabled={owned.has(hit.external_id) || add.isPending}
              onClick={() =>
                add.mutate(
                  { mediaType: type, externalId: hit.external_id, status },
                  { onSuccess: onClose },
                )
              }
            >
              {owned.has(hit.external_id) ? "In library" : "Add"}
            </Button>
          </li>
        ))}
      </ul>
      {add.error && <p className="text-sm text-destructive">{errorMessage(add.error)}</p>}
    </>
  );
}
