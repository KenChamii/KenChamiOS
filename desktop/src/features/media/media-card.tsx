import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { LibraryEntry } from "@/lib/api/media";
import { Cover } from "./cover";
import { progressText, STATUS_LABELS } from "./meta";
import { PlusOneButton } from "./plus-one-button";

type MediaItemProps = { entry: LibraryEntry; onOpen: () => void };

export function MediaCard({ entry, onOpen }: MediaItemProps) {
  return (
    <li className="flex flex-col gap-2">
      <button type="button" onClick={onOpen} className="flex flex-col gap-2 text-left">
        <Cover title={entry.title} path={entry.cover_path} url={entry.cover_url} />
        <span className="line-clamp-2 text-sm font-medium">{entry.title}</span>
      </button>
      {entry.progress_total !== null && (
        <Progress value={entry.progress} max={entry.progress_total} />
      )}
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 flex-col">
          <Badge className="w-fit">{STATUS_LABELS[entry.media_type][entry.status]}</Badge>
          <span className="text-xs text-muted-foreground">{progressText(entry)}</span>
        </div>
        <PlusOneButton entry={entry} />
      </div>
    </li>
  );
}
