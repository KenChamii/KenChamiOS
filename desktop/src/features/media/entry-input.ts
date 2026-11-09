import type { EntryInput, LibraryEntry, MediaStatus } from "@/lib/api/media";
import { fromDateInput, toDateInput } from "@/lib/dates";
import { formatTags, parseTags } from "@/lib/tags";

/** The edit form as text, so half typed values are possible. */
export type EntryDraft = {
  status: MediaStatus;
  total: string;
  rating: string;
  notes: string;
  tags: string;
  startedAt: string;
  finishedAt: string;
  platform: string;
};

export function draftOf(entry: LibraryEntry): EntryDraft {
  return {
    status: entry.status,
    total: entry.progress_total === null ? "" : String(entry.progress_total),
    rating: entry.rating === null ? "" : String(entry.rating),
    notes: entry.notes,
    tags: formatTags(entry.tags),
    startedAt: toDateInput(entry.started_at),
    finishedAt: toDateInput(entry.finished_at),
    platform: entry.platform ?? "",
  };
}

/** A whole number in range from a text field; "" is no value, anything else invalid is undefined. */
function wholeNumber(text: string, min: number, max: number): number | null | undefined {
  if (text.trim() === "") return null;
  const value = Number(text);
  return Number.isInteger(value) && value >= min && value <= max ? value : undefined;
}

/** The input to save, or null while the form has a rating or total that is not valid. */
export function inputOf(draft: EntryDraft): EntryInput | null {
  const rating = wholeNumber(draft.rating, 1, 10);
  const total = wholeNumber(draft.total, 0, 1_000_000);
  if (rating === undefined || total === undefined) return null;
  return {
    status: draft.status,
    progress_total: total,
    rating,
    notes: draft.notes,
    tags: parseTags(draft.tags),
    started_at: fromDateInput(draft.startedAt),
    finished_at: fromDateInput(draft.finishedAt),
    platform: draft.platform.trim() || null,
  };
}
