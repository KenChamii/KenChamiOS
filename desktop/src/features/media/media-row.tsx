import { Badge } from "@/components/ui/badge";
import type { LibraryEntry } from "@/lib/api/media";
import { Cover } from "./cover";
import { progressText, STATUS_LABELS, TYPE_LABELS } from "./meta";
import { PlusOneButton } from "./plus-one-button";

export function MediaRow({ entry, onOpen }: { entry: LibraryEntry; onOpen: () => void }) {
  return (
    <li className="flex items-center gap-3 py-2">
      <button
        type="button"
        onClick={onOpen}
        className="flex min-w-0 flex-1 items-center gap-3 text-left"
      >
        <Cover title={entry.title} path={entry.cover_path} url={entry.cover_url} className="w-10" />
        <span className="flex min-w-0 flex-col">
          <span className="truncate text-sm font-medium">{entry.title}</span>
          <span className="text-xs text-muted-foreground">
            {TYPE_LABELS[entry.media_type]}
            {entry.year !== null && ` · ${entry.year}`}
            {entry.rating !== null && ` · ${entry.rating}/10`}
          </span>
        </span>
      </button>
      <Badge>{STATUS_LABELS[entry.media_type][entry.status]}</Badge>
      <span className="w-20 text-right text-sm text-muted-foreground">{progressText(entry)}</span>
      <PlusOneButton entry={entry} />
    </li>
  );
}
