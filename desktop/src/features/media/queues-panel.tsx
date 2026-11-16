import { useState } from "react";
import { Card } from "@/components/ui/card";
import { ChipGroup } from "@/components/ui/chips";
import type { LibraryEntry, MediaType } from "@/lib/api/media";
import { Cover } from "./cover";
import { TYPES } from "./meta";
import { continueQueue, nextUnit, queueTypes, upNextQueue } from "./queues";

type Props = { entries: LibraryEntry[]; onOpen: (id: string) => void };

/** Continue and up-next lists, one media type at a time. */
export function QueuesPanel({ entries, onOpen }: Props) {
  const types = queueTypes(
    entries,
    TYPES.map((type) => type.id),
  );
  const [chosen, setChosen] = useState<MediaType | null>(null);
  const type = chosen !== null && types.includes(chosen) ? chosen : types[0];
  if (!type) return null;

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-semibold tracking-tight">Continue and up next</h2>
        <ChipGroup
          aria-label="Media type"
          options={TYPES.filter((item) => types.includes(item.id)).map((item) => ({
            value: item.id,
            label: item.label,
          }))}
          value={type}
          onChange={setChosen}
        />
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <Queue title="Continue" entries={continueQueue(entries, type)} onOpen={onOpen} />
        <Queue title="Up next" entries={upNextQueue(entries, type)} onOpen={onOpen} />
      </div>
    </Card>
  );
}

function Queue({ title, entries, onOpen }: { title: string } & Props) {
  return (
    <div className="flex flex-col gap-2">
      <h3 className="text-label text-muted-foreground">{title}</h3>
      {entries.length === 0 ? (
        <p className="text-sm text-muted-foreground">Nothing here.</p>
      ) : (
        <ul className="flex flex-col gap-1">
          {entries.map((entry) => (
            <li key={entry.id}>
              <button
                type="button"
                onClick={() => onOpen(entry.id)}
                className="flex w-full items-center gap-3 rounded-xl p-1.5 text-left hover:bg-muted"
              >
                <Cover
                  title={entry.title}
                  path={entry.cover_path}
                  url={entry.cover_url}
                  mediaType={entry.media_type}
                  className="w-8"
                />
                <span className="flex min-w-0 flex-col">
                  <span className="truncate text-sm font-medium">{entry.title}</span>
                  {nextUnit(entry) && (
                    <span className="text-xs text-muted-foreground">{nextUnit(entry)}</span>
                  )}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
