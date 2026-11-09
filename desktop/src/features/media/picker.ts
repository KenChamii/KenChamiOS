import type { LibraryEntry, MediaType } from "@/lib/api/media";

export type PickerFilter = {
  type: MediaType | "all";
  genre: string;
  tag: string;
  /** most episodes or chapters still to go; null for no limit */
  maxLeft: number | null;
};

export const NO_PICKER_FILTER: PickerFilter = { type: "all", genre: "", tag: "", maxLeft: null };

/** Units still to go, or null when the total is not known. */
export function remaining(entry: Pick<LibraryEntry, "progress" | "progress_total">) {
  return entry.progress_total === null ? null : Math.max(entry.progress_total - entry.progress, 0);
}

/** The planned entries that fit the filter; with a limit, entries of unknown length are left out. */
export function backlog(entries: LibraryEntry[], filter: PickerFilter): LibraryEntry[] {
  return entries.filter((entry) => {
    if (entry.status !== "planned") return false;
    if (filter.type !== "all" && entry.media_type !== filter.type) return false;
    if (filter.genre && !entry.genres.includes(filter.genre)) return false;
    if (filter.tag && !entry.tags.includes(filter.tag)) return false;
    if (filter.maxLeft === null) return true;
    const left = remaining(entry);
    return left !== null && left <= filter.maxLeft;
  });
}

/** One random candidate, never the current one while there is another choice. */
export function pickNext(
  candidates: LibraryEntry[],
  currentId: string | null,
  random: () => number = Math.random,
): LibraryEntry | null {
  const others = candidates.filter((entry) => entry.id !== currentId);
  const pool = others.length > 0 ? others : candidates;
  return pool.length === 0 ? null : (pool[Math.floor(random() * pool.length)] ?? null);
}
