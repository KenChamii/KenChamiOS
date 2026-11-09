import { Link } from "react-router-dom";
import { Cover } from "@/features/media/cover";
import { PlusOneButton } from "@/features/media/plus-one-button";
import { useMedia } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";
import { stagger } from "@/lib/motion";
import { continueItems, nextLabel } from "./continue-items";

/** What is in progress, as a row of covers; hidden when nothing is. */
export function ContinueRow() {
  const media = useMedia();
  const items = continueItems(media.data ?? []);
  if (media.error) return <p className="text-sm text-destructive">{errorMessage(media.error)}</p>;
  if (items.length === 0) return null;

  return (
    <section className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Continue</h2>
      <ul className="-mx-2 flex gap-5 overflow-x-auto px-2 pb-3 pt-1">
        {items.map((entry, index) => (
          <li
            key={entry.id}
            className="stagger-in flex w-36 shrink-0 flex-col gap-2"
            style={stagger(index)}
          >
            <Link to={`/media?open=${entry.id}`} className="lift flex flex-col gap-2 rounded-xl">
              <Cover
                title={entry.title}
                path={entry.cover_path}
                url={entry.cover_url}
                mediaType={entry.media_type}
                className="w-full shadow-soft"
              />
              <span className="flex flex-col">
                <span className="truncate text-sm font-medium">{entry.title}</span>
                <span className="text-xs text-muted-foreground">{nextLabel(entry)}</span>
              </span>
            </Link>
            <PlusOneButton entry={entry} />
          </li>
        ))}
      </ul>
    </section>
  );
}
