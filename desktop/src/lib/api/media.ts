import { invoke } from "@tauri-apps/api/core";

export type MediaType = "anime" | "manga" | "game";
export type MediaStatus = "in_progress" | "planned" | "completed" | "on_hold" | "dropped";
export type MediaSource = "anilist" | "rawg" | "manual";

export type LibraryEntry = {
  id: string;
  created_at: number;
  updated_at: number;
  media_item_id: string;
  media_type: MediaType;
  title: string;
  cover_url: string | null;
  cover_cache_path: string | null;
  /** absolute path of the cached cover on this device */
  cover_path: string | null;
  year: number | null;
  genres: string[];
  synopsis: string;
  source: MediaSource;
  external_id: string | null;
  status: MediaStatus;
  progress: number;
  progress_total: number | null;
  rating: number | null;
  notes: string;
  tags: string[];
  started_at: number | null;
  finished_at: number | null;
  platform: string | null;
};

export type ItemDetails = {
  title: string;
  cover_url: string | null;
  year: number | null;
  genres: string[];
  synopsis: string;
  progress_total: number | null;
};

export type SearchResult = ItemDetails & { external_id: string };

export type ManualInput = ItemDetails & { media_type: MediaType; status: MediaStatus };

export type EntryInput = {
  status: MediaStatus;
  progress_total: number | null;
  rating: number | null;
  notes: string;
  tags: string[];
  started_at: number | null;
  finished_at: number | null;
  platform: string | null;
};

export function listMedia() {
  return invoke<LibraryEntry[]>("media_list");
}

export function searchMedia(query: string, mediaType: MediaType) {
  return invoke<SearchResult[]>("media_search", { query, mediaType });
}

export function addMedia(mediaType: MediaType, externalId: string, status: MediaStatus) {
  return invoke<LibraryEntry>("media_add", { mediaType, externalId, status });
}

export function addManualMedia(input: ManualInput) {
  return invoke<LibraryEntry>("media_add_manual", { input });
}

export function updateMedia(id: string, input: EntryInput) {
  return invoke<LibraryEntry>("media_update", { id, input });
}

export function setMediaProgress(id: string, value: number) {
  return invoke<LibraryEntry>("media_set_progress", { id, value });
}

export function deleteMedia(id: string) {
  return invoke<void>("media_delete", { id });
}

export function gamesKeyStatus() {
  return invoke<boolean>("games_key_status");
}

export function setGamesKey(key: string) {
  return invoke<void>("games_key_set", { key });
}

export function clearGamesKey() {
  return invoke<void>("games_key_clear");
}
