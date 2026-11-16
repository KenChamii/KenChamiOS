import type { LibraryEntry, MediaType } from "@/lib/api/media";

export type YearReview = {
  finished: number;
  /** finished titles per month, January first */
  byMonth: number[];
  gameHours: number;
  episodes: number;
  chapters: number;
  topGenres: { genre: string; count: number }[];
  highestRated: LibraryEntry[];
};

const TOP_COUNT = 5;

function finishedYear(entry: LibraryEntry): number | null {
  if (entry.status !== "completed" || entry.finished_at === null) return null;
  return new Date(entry.finished_at).getFullYear();
}

/** Years that have at least one finished title, newest first. */
export function reviewYears(entries: LibraryEntry[]): number[] {
  const years = new Set(entries.map(finishedYear).filter((year) => year !== null));
  return [...years].sort((a, b) => b - a);
}

function sum(entries: LibraryEntry[], type: MediaType): number {
  return entries
    .filter((entry) => entry.media_type === type)
    .reduce((total, entry) => total + entry.progress, 0);
}

/** What the library says about titles finished in `year`. */
export function yearReview(entries: LibraryEntry[], year: number): YearReview {
  const finished = entries.filter((entry) => finishedYear(entry) === year);
  const byMonth = Array.from({ length: 12 }, () => 0);
  const genreCounts = new Map<string, number>();
  for (const entry of finished) {
    byMonth[new Date(entry.finished_at ?? 0).getMonth()] += 1;
    for (const genre of entry.genres) genreCounts.set(genre, (genreCounts.get(genre) ?? 0) + 1);
  }
  const topGenres = [...genreCounts]
    .map(([genre, count]) => ({ genre, count }))
    .sort((a, b) => b.count - a.count || a.genre.localeCompare(b.genre))
    .slice(0, TOP_COUNT);
  const highestRated = finished
    .filter((entry) => entry.rating !== null)
    .sort((a, b) => (b.rating ?? 0) - (a.rating ?? 0) || a.title.localeCompare(b.title))
    .slice(0, TOP_COUNT);
  return {
    finished: finished.length,
    byMonth,
    gameHours: sum(finished, "game"),
    episodes: sum(finished, "anime") + sum(finished, "show"),
    chapters: sum(finished, "manga"),
    topGenres,
    highestRated,
  };
}
