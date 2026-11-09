import { Badge } from "@/components/ui/badge";
import type { LibraryEntry } from "@/lib/api/media";
import { stagger } from "@/lib/motion";
import { Cover } from "./cover";
import { progressText, STATUS_LABELS } from "./meta";
import { PlusOneButton } from "./plus-one-button";
import { TypeBadge } from "./type-badge";

type MediaRowProps = { entry: LibraryEntry; index: number; onOpen: () => void };

export function MediaRow({ entry, index, onOpen }: MediaRowProps) {
  return (
    <li className="stagger-in flex items-center gap-3 py-2" style={stagger(index)}>
      <button
        type="button"
        onClick={onOpen}
        className="flex min-w-0 flex-1 items-center gap-3 text-left"
      >
        <Cover
          title={entry.title}
          path={entry.cover_path}
          url={entry.cover_url}
          mediaType={entry.media_type}
          className="w-10"
        />
        <span className="flex min-w-0 flex-col">
          <span className="truncate text-sm font-medium">{entry.title}</span>
          {(entry.year !== null || entry.rating !== null) && (
            <span className="text-xs text-muted-foreground">
              {[entry.year, entry.rating !== null && `${entry.rating}/10`]
                .filter(Boolean)
                .join(" · ")}
            </span>
          )}
        </span>
      </button>
      <TypeBadge type={entry.media_type} />
      <Badge>{STATUS_LABELS[entry.media_type][entry.status]}</Badge>
      <span className="w-20 text-right text-sm text-muted-foreground">{progressText(entry)}</span>
      <PlusOneButton entry={entry} />
    </li>
  );
}
