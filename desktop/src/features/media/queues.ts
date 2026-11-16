import type { LibraryEntry, MediaType } from "@/lib/api/media";

const NEXT_COUNT = 8;
const UNIT_LABEL: Partial<Record<MediaType, string>> = {
  anime: "Episode",
  show: "Episode",
  manga: "Chapter",
};

/** Started titles of one type, the one touched last first. */
export function continueQueue(entries: LibraryEntry[], type: MediaType): LibraryEntry[] {
  return entries
    .filter((entry) => entry.media_type === type && entry.status === "in_progress")
    .sort((a, b) => b.updated_at - a.updated_at);
}

/** Planned titles of one type, the one added first first. */
export function upNextQueue(entries: LibraryEntry[], type: MediaType): LibraryEntry[] {
  return entries
    .filter((entry) => entry.media_type === type && entry.status === "planned")
    .sort((a, b) => a.created_at - b.created_at)
    .slice(0, NEXT_COUNT);
}

/** What to open next in a started title, for example "Episode 5"; null for types without units. */
export function nextUnit(entry: LibraryEntry): string | null {
  const unit = UNIT_LABEL[entry.media_type];
  if (!unit) return null;
  const next = entry.progress + 1;
  if (entry.progress_total !== null && next > entry.progress_total) return null;
  return `${unit} ${next}`;
}

/** The types that have something in either queue, in the library's order. */
export function queueTypes(entries: LibraryEntry[], order: MediaType[]): MediaType[] {
  return order.filter(
    (type) => continueQueue(entries, type).length > 0 || upNextQueue(entries, type).length > 0,
  );
}
