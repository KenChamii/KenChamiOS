import { ArrowDownUp, Dices, Plus } from "lucide-react";
import { useState } from "react";
import { PageHeader } from "@/components/layout/page-header";
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
import { PickerDialog } from "@/features/media/picker-dialog";
import { TransferDialog } from "@/features/media/transfer-dialog";
import { SearchDialog } from "@/features/media/search-dialog";
import { useMedia } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";

export function MediaPage() {
  const media = useMedia();
  const [filter, setFilter] = useState(NO_FILTER);
  const [view, setView] = useState<ViewMode>("grid");
  const [searchOpen, setSearchOpen] = useState(false);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [openId, setOpenId] = useState<string | null>(null);

  const entries = media.data ?? [];
  const shown = filterEntries(entries, filter);
  // read from the live list, so the sheet follows progress changes
  const opened = entries.find((entry) => entry.id === openId) ?? null;
  const isFiltering = isFiltered(filter);

  return (
    <div className="flex flex-col gap-6">
      <PageHeader title="Media" subtitle="Your library and what to watch, read or play next">
        <Button variant="outline" onClick={() => setTransferOpen(true)}>
          <ArrowDownUp size={16} />
          Import / export
        </Button>
        <Button variant="outline" onClick={() => setPickerOpen(true)}>
          <Dices size={16} />
          What next?
        </Button>
        <Button onClick={() => setSearchOpen(true)}>
          <Plus size={16} />
          Add
        </Button>
      </PageHeader>

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
      <TransferDialog open={transferOpen} onClose={() => setTransferOpen(false)} />
      <PickerDialog
        entries={entries}
        open={pickerOpen}
        onClose={() => setPickerOpen(false)}
        onOpenEntry={setOpenId}
      />
      <EntrySheet entry={opened} onClose={() => setOpenId(null)} />
    </div>
  );
}
