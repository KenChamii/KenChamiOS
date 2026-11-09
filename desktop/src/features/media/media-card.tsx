import type { LibraryEntry } from "@/lib/api/media";
import { stagger } from "@/lib/motion";
import { Cover } from "./cover";
import { progressText, STATUS_LABELS, TYPE_ICONS, TYPE_LABELS } from "./meta";
import { PlusOneButton } from "./plus-one-button";

type MediaItemProps = { entry: LibraryEntry; index: number; onOpen: () => void };

export function MediaCard({ entry, index, onOpen }: MediaItemProps) {
  const TypeIcon = TYPE_ICONS[entry.media_type];
  const percent =
    entry.progress_total && entry.progress_total > 0
      ? Math.min(100, (entry.progress / entry.progress_total) * 100)
      : null;

  return (
    <li className="group stagger-in relative" style={stagger(index)}>
      <button
        type="button"
        onClick={onOpen}
        className="lift relative block w-full overflow-hidden rounded-2xl text-left shadow-soft focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <Cover
          title={entry.title}
          path={entry.cover_path}
          url={entry.cover_url}
          mediaType={entry.media_type}
          className="rounded-none"
        />
        <span
          className="absolute left-2 top-2 flex h-7 w-7 items-center justify-center rounded-lg bg-card/85 text-media backdrop-blur"
          title={TYPE_LABELS[entry.media_type]}
        >
          <TypeIcon size={15} aria-hidden />
        </span>
        <span className="absolute inset-x-0 bottom-0 flex flex-col gap-0.5 bg-linear-to-t from-black/80 via-black/45 to-transparent p-3 pt-12 text-white">
          <span className="line-clamp-2 text-sm font-semibold leading-snug">{entry.title}</span>
          <span className="text-xs text-white/75">
            {STATUS_LABELS[entry.media_type][entry.status]} · {progressText(entry)}
          </span>
        </span>
        {percent !== null && (
          <span className="absolute inset-x-0 bottom-0 h-1 bg-white/20">
            <span className="block h-full bg-media" style={{ width: `${percent}%` }} />
          </span>
        )}
      </button>
      <div className="absolute right-2 top-2 opacity-0 transition-opacity duration-(--duration-fast) group-focus-within:opacity-100 group-hover:opacity-100">
        <PlusOneButton entry={entry} />
      </div>
    </li>
  );
}
