import { useState } from "react";
import { Card } from "@/components/ui/card";
import { useUpcoming } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";
import { Cover } from "./cover";
import { groupUpcoming } from "./upcoming";

function formatWhen(ms: number): string {
  return new Date(ms).toLocaleString(undefined, {
    weekday: "short",
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Next episodes of followed anime and shows. Hidden when nothing is airing. */
export function UpcomingPanel({ onOpen }: { onOpen: (id: string) => void }) {
  const upcoming = useUpcoming();
  const [now] = useState(() => Date.now());
  const groups = groupUpcoming(upcoming.data ?? [], now);

  if (upcoming.error) {
    return <p className="text-sm text-muted-foreground">{errorMessage(upcoming.error)}</p>;
  }
  if (groups.length === 0) return null;

  return (
    <Card className="flex flex-col gap-4">
      <h2 className="font-semibold tracking-tight">Coming up</h2>
      {groups.map((group) => (
        <div key={group.label} className="flex flex-col gap-1">
          <h3 className="text-label text-muted-foreground">{group.label}</h3>
          <ul className="flex flex-col">
            {group.releases.map(({ entry, episode, air_at_ms }) => (
              <li key={`${entry.id}-${episode}`}>
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
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate text-sm font-medium">{entry.title}</span>
                    <span className="text-xs text-muted-foreground">Episode {episode}</span>
                  </span>
                  <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
                    {formatWhen(air_at_ms)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </Card>
  );
}
