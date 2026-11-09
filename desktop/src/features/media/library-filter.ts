import type { LibraryEntry, MediaStatus, MediaType } from "@/lib/api/media";

export type SortKey = "recent" | "added" | "title" | "rating";

export type LibraryFilter = {
  search: string;
  type: MediaType | "all";
  status: MediaStatus | "all";
  genre: string;
  tag: string;
  sort: SortKey;
};

export const NO_FILTER: LibraryFilter = {
  search: "",
  type: "all",
  status: "all",
  genre: "",
  tag: "",
  sort: "recent",
};

export function filterEntries(entries: LibraryEntry[], filter: LibraryFilter): LibraryEntry[] {
  const needle = filter.search.trim().toLowerCase();
  const matching = entries.filter(
    (entry) =>
      (filter.type === "all" || entry.media_type === filter.type) &&
      (filter.status === "all" || entry.status === filter.status) &&
      (!filter.genre || entry.genres.includes(filter.genre)) &&
      (!filter.tag || entry.tags.includes(filter.tag)) &&
      (!needle || entry.title.toLowerCase().includes(needle)),
  );
  return matching.sort(COMPARERS[filter.sort]);
}

const COMPARERS: Record<SortKey, (a: LibraryEntry, b: LibraryEntry) => number> = {
  recent: (a, b) => b.updated_at - a.updated_at,
  added: (a, b) => b.created_at - a.created_at,
  title: (a, b) => a.title.localeCompare(b.title),
  // unrated entries go last
  rating: (a, b) => (b.rating ?? 0) - (a.rating ?? 0) || a.title.localeCompare(b.title),
};

/** Sorted values of a list field that occur in the library, for the filter menus. */
export function distinctValues(entries: LibraryEntry[], field: "genres" | "tags"): string[] {
  return [...new Set(entries.flatMap((entry) => entry[field]))].sort((a, b) => a.localeCompare(b));
}

/** True when something narrows the list; the sort order does not. */
export function isFiltered(filter: LibraryFilter): boolean {
  return (
    filter.search.trim() !== "" ||
    filter.type !== "all" ||
    filter.status !== "all" ||
    filter.genre !== "" ||
    filter.tag !== ""
  );
}
