import type { ManualInput, MediaStatus, MediaType } from "@/lib/api/media";

type ManualDraft = {
  type: MediaType;
  status: MediaStatus;
  title: string;
  year: string;
  total: string;
  genres: string[];
};

function optionalInteger(text: string): number | null | undefined {
  if (text.trim() === "") return null;
  const value = Number(text);
  return Number.isInteger(value) && value >= 0 ? value : undefined;
}

/** The input to add, or null while the title is blank or the year or total is not a number. */
export function manualInputOf(draft: ManualDraft): ManualInput | null {
  const title = draft.title.trim();
  const year = optionalInteger(draft.year);
  const total = draft.type === "game" ? null : optionalInteger(draft.total);
  if (!title || year === undefined || total === undefined) return null;
  return {
    media_type: draft.type,
    status: draft.status,
    title,
    cover_url: null,
    year,
    genres: draft.genres,
    synopsis: "",
    progress_total: total,
  };
}
