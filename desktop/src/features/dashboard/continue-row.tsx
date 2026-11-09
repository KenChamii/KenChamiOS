import { Card } from "@/components/ui/card";
import { Cover } from "@/features/media/cover";
import { TYPE_LABELS } from "@/features/media/meta";
import { PlusOneButton } from "@/features/media/plus-one-button";
import { useMedia } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";
import { continueItems, nextLabel } from "./continue-items";

export function ContinueRow() {
  const media = useMedia();
  const items = continueItems(media.data ?? []);

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Continue</h2>
      {media.error && <p className="text-sm text-destructive">{errorMessage(media.error)}</p>}
      {items.length === 0 && !media.error && (
        <p className="text-sm text-muted-foreground">
          Nothing in progress. Start something from the Media page.
        </p>
      )}
      <ul className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
        {items.map((entry) => (
          <li key={entry.id} className="flex items-center gap-3">
            <Cover
              title={entry.title}
              path={entry.cover_path}
              url={entry.cover_url}
              className="w-12 shrink-0"
            />
            <div className="flex min-w-0 flex-1 flex-col">
              <span className="truncate text-sm font-medium">{entry.title}</span>
              <span className="text-xs text-muted-foreground">
                {TYPE_LABELS[entry.media_type]} · {nextLabel(entry)}
              </span>
            </div>
            <PlusOneButton entry={entry} />
          </li>
        ))}
      </ul>
    </Card>
  );
}
