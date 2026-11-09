import type { LibraryEntry, MediaStatus, MediaType } from "@/lib/api/media";

export const TYPES: { id: MediaType; label: string }[] = [
  { id: "anime", label: "Anime" },
  { id: "manga", label: "Manga" },
  { id: "game", label: "Games" },
];

export const TYPE_LABELS: Record<MediaType, string> = {
  anime: "Anime",
  manga: "Manga",
  game: "Game",
};

export const STATUSES: MediaStatus[] = [
  "in_progress",
  "planned",
  "completed",
  "on_hold",
  "dropped",
];

export const STATUS_LABELS: Record<MediaType, Record<MediaStatus, string>> = {
  anime: {
    in_progress: "Watching",
    planned: "Plan to watch",
    completed: "Completed",
    on_hold: "On hold",
    dropped: "Dropped",
  },
  manga: {
    in_progress: "Reading",
    planned: "Plan to read",
    completed: "Completed",
    on_hold: "On hold",
    dropped: "Dropped",
  },
  game: {
    in_progress: "Playing",
    planned: "Backlog",
    completed: "Completed",
    on_hold: "On hold",
    dropped: "Dropped",
  },
};

/** What the progress counter counts, per type. */
export const PROGRESS_LABELS: Record<MediaType, string> = {
  anime: "Episode",
  manga: "Chapter",
  game: "Hours played",
};

export const SOURCE_LINKS: Record<MediaType, { name: string; url: string }> = {
  anime: { name: "AniList", url: "https://anilist.co" },
  manga: { name: "AniList", url: "https://anilist.co" },
  game: { name: "RAWG", url: "https://rawg.io" },
};

export const GENERIC_STATUS_LABELS: Record<MediaStatus, string> = {
  in_progress: "In progress",
  planned: "Planned",
  completed: "Completed",
  on_hold: "On hold",
  dropped: "Dropped",
};

export const SORT_LABELS = {
  recent: "Recently updated",
  added: "Recently added",
  title: "Title",
  rating: "Rating",
} as const;

/** "12 / 28", "12" when the total is unknown, "12 h" for games. */
export function progressText(
  entry: Pick<LibraryEntry, "media_type" | "progress" | "progress_total">,
) {
  if (entry.media_type === "game") return `${entry.progress} h`;
  return entry.progress_total === null
    ? String(entry.progress)
    : `${entry.progress} / ${entry.progress_total}`;
}
