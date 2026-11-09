import type { LibraryEntry } from "@/lib/api/media";

/** A planned anime entry; tests override what they care about. */
export function testEntry(overrides: Partial<LibraryEntry> = {}): LibraryEntry {
  return {
    id: "e",
    created_at: 0,
    updated_at: 0,
    media_item_id: "i",
    media_type: "anime",
    title: "Untitled",
    cover_url: null,
    cover_cache_path: null,
    cover_path: null,
    year: null,
    genres: [],
    synopsis: "",
    source: "manual",
    external_id: null,
    status: "planned",
    progress: 0,
    progress_total: null,
    rating: null,
    notes: "",
    tags: [],
    started_at: null,
    finished_at: null,
    platform: null,
    ...overrides,
  };
}
