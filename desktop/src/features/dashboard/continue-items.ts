import type { LibraryEntry } from "@/lib/api/media";

const MAX_ITEMS = 6;
const CONTINUABLE = ["anime", "manga", "show", "game"] as const;

/** What is in progress, most recently touched first. Movies and music have nothing to continue. */
export function continueItems(entries: LibraryEntry[]): LibraryEntry[] {
  return entries
    .filter(
      (entry) =>
        entry.status === "in_progress" &&
        (CONTINUABLE as readonly string[]).includes(entry.media_type),
    )
    .sort((a, b) => b.updated_at - a.updated_at)
    .slice(0, MAX_ITEMS);
}

/** "Episode 13", "Chapter 141", or the hours so far for a game. */
export function nextLabel(entry: Pick<LibraryEntry, "media_type" | "progress">): string {
  switch (entry.media_type) {
    case "manga":
      return `Chapter ${entry.progress + 1}`;
    case "game":
      return entry.progress === 0 ? "Just started" : `${entry.progress} h played`;
    default:
      return `Episode ${entry.progress + 1}`;
  }
}
