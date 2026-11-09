import { invoke } from "@tauri-apps/api/core";
import type { ItemDetails, MediaSource, MediaStatus, MediaType } from "./media";

export type ImportFormat = "csv" | "mal";

export type PreviewRow = ItemDetails & {
  line: number;
  media_type: MediaType;
  status: MediaStatus;
  progress: number;
  rating: number | null;
  notes: string;
  tags: string[];
  started_at: number | null;
  finished_at: number | null;
  platform: string | null;
  source: MediaSource;
  external_id: string | null;
  duplicate: boolean;
};

export type RejectedRow = { line: number; reason: string };

export type ImportPreview = {
  rows: PreviewRow[];
  rejected: RejectedRow[];
  new_count: number;
  duplicate_count: number;
};

export type ImportSummary = { added: number; duplicates: number; rejected: number };

export function previewImport(path: string, format: ImportFormat) {
  return invoke<ImportPreview>("media_import_preview", { path, format });
}

export function confirmImport(path: string, format: ImportFormat) {
  return invoke<ImportSummary>("media_import_confirm", { path, format });
}

export function exportExists(folder: string) {
  return invoke<boolean>("media_export_exists", { folder });
}

export function exportCsv(folder: string, overwrite: boolean) {
  return invoke<string>("media_export_csv", { folder, overwrite });
}
