import { invoke } from "@tauri-apps/api/core";

export type BackupFile = {
  name: string;
  path: string;
  size_bytes: number;
  created_at: number | null;
};

export type BackupStatus = {
  folder: string;
  /** false while the default folder next to the database is used */
  custom_folder: boolean;
  keep: number;
  last_at: number | null;
  last_error: string | null;
  /** newest first */
  copies: BackupFile[];
};

export function getBackupStatus() {
  return invoke<BackupStatus>("backups_status");
}

/** A blank folder means the default one. */
export function configureBackups(folder: string | null, keep: number) {
  return invoke<BackupStatus>("backups_configure", { folder, keep });
}

export function runBackup() {
  return invoke<BackupStatus>("backups_run");
}

/** Replaces the database with the backup and restarts the app. */
export function restoreBackup(path: string) {
  return invoke<void>("backups_restore", { path });
}
