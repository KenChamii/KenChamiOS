import { Plus } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { EntrySheet } from "@/features/media/entry-sheet";
import {
  distinctValues,
  filterEntries,
  isFiltered,
  NO_FILTER,
} from "@/features/media/library-filter";
import { LibraryToolbar, type ViewMode } from "@/features/media/library-toolbar";
import { MediaCard } from "@/features/media/media-card";
import { MediaRow } from "@/features/media/media-row";
import { SearchDialog } from "@/features/media/search-dialog";
import { useMedia } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";

export function MediaPage() {
  const media = useMedia();
  const [filter, setFilter] = useState(NO_FILTER);
  const [view, setView] = useState<ViewMode>("grid");
  const [searchOpen, setSearchOpen] = useState(false);
  const [openId, setOpenId] = useState<string | null>(null);

  const entries = media.data ?? [];
  const shown = filterEntries(entries, filter);
  // read from the live list, so the sheet follows progress changes
  const opened = entries.find((entry) => entry.id === openId) ?? null;
  const isFiltering = isFiltered(filter);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Media</h1>
        <Button onClick={() => setSearchOpen(true)}>
          <Plus size={16} />
          Add
        </Button>
      </div>

      <LibraryToolbar
        filter={filter}
        onFilter={setFilter}
        genres={distinctValues(entries, "genres")}
        tags={distinctValues(entries, "tags")}
        view={view}
        onView={setView}
      />

      {media.error ? (
        <p className="text-sm text-destructive">{errorMessage(media.error)}</p>
      ) : shown.length === 0 ? (
        <div className="flex flex-col items-center gap-3 py-16 text-center">
          <p className="text-muted-foreground">
            {isFiltering ? "Nothing matches these filters." : "Your library is empty."}
          </p>
          {!isFiltering && (
            <Button onClick={() => setSearchOpen(true)}>Add your first title</Button>
          )}
        </div>
      ) : view === "grid" ? (
        <ul className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-5 xl:grid-cols-6">
          {shown.map((entry) => (
            <MediaCard key={entry.id} entry={entry} onOpen={() => setOpenId(entry.id)} />
          ))}
        </ul>
      ) : (
        <ul className="flex flex-col divide-y divide-border">
          {shown.map((entry) => (
            <MediaRow key={entry.id} entry={entry} onOpen={() => setOpenId(entry.id)} />
          ))}
        </ul>
      )}

      <SearchDialog entries={entries} open={searchOpen} onClose={() => setSearchOpen(false)} />
      <EntrySheet entry={opened} onClose={() => setOpenId(null)} />
    </div>
  );
}
