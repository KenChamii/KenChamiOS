import {
  BookOpen,
  Clapperboard,
  Disc3,
  Gamepad2,
  MicVocal,
  MonitorPlay,
  Tv,
  type LucideIcon,
} from "lucide-react";
import type { LibraryEntry, MediaStatus, MediaType } from "@/lib/api/media";

export const TYPE_ICONS: Record<MediaType, LucideIcon> = {
  anime: Tv,
  manga: BookOpen,
  game: Gamepad2,
  movie: Clapperboard,
  show: MonitorPlay,
  album: Disc3,
  artist: MicVocal,
};

export const TYPES: { id: MediaType; label: string }[] = [
  { id: "anime", label: "Anime" },
  { id: "manga", label: "Manga" },
  { id: "game", label: "Games" },
  { id: "movie", label: "Movies" },
  { id: "show", label: "Shows" },
  { id: "album", label: "Albums" },
  { id: "artist", label: "Artists" },
];

export const TYPE_LABELS: Record<MediaType, string> = {
  anime: "Anime",
  manga: "Manga",
  game: "Game",
  movie: "Movie",
  show: "Show",
  album: "Album",
  artist: "Artist",
};

export const STATUSES: MediaStatus[] = [
  "in_progress",
  "planned",
  "completed",
  "on_hold",
  "dropped",
];

const WATCH_LABELS: Record<MediaStatus, string> = {
  in_progress: "Watching",
  planned: "Watchlist",
  completed: "Watched",
  on_hold: "On hold",
  dropped: "Dropped",
};

const LISTEN_LABELS: Record<MediaStatus, string> = {
  in_progress: "Listening",
  planned: "To try",
  completed: "Favorite",
  on_hold: "On hold",
  dropped: "Dropped",
};

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
  movie: WATCH_LABELS,
  show: WATCH_LABELS,
  album: LISTEN_LABELS,
  artist: LISTEN_LABELS,
};

/** What the progress counter counts, per type. */
export const PROGRESS_LABELS: Record<MediaType, string> = {
  anime: "Episode",
  manga: "Chapter",
  game: "Hours played",
  movie: "Watched",
  show: "Episode",
  album: "Listens",
  artist: "Listens",
};

/** The label of the editable total, or null when the type has none. */
export const TOTAL_LABELS: Record<MediaType, string | null> = {
  anime: "Total episodes",
  manga: "Total chapters",
  game: null,
  movie: null,
  show: "Total episodes",
  album: null,
  artist: null,
};

/** What the progress counter counts, in plural, for summaries. */
export const UNIT_NAMES: Record<MediaType, string> = {
  anime: "episodes",
  manga: "chapters",
  game: "hours",
  movie: "movies",
  show: "episodes",
  album: "listens",
  artist: "listens",
};

export const SOURCE_LINKS: Record<MediaType, { name: string; url: string }> = {
  anime: { name: "AniList", url: "https://anilist.co" },
  manga: { name: "AniList", url: "https://anilist.co" },
  game: { name: "RAWG", url: "https://rawg.io" },
  movie: { name: "TMDB", url: "https://www.themoviedb.org" },
  show: { name: "TMDB", url: "https://www.themoviedb.org" },
  album: { name: "MusicBrainz", url: "https://musicbrainz.org" },
  artist: { name: "MusicBrainz", url: "https://musicbrainz.org" },
};

/** Types whose search needs a key or token saved in Settings. */
export const KEYED_TYPES: MediaType[] = ["game", "movie", "show"];

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
