-- desktop-only, never synced: each row describes one file in this device's Obsidian vault.
-- hash is the file as last synced, app_hash the app side of it, so either side's edit is seen
CREATE TABLE vault_files (
    path      TEXT PRIMARY KEY NOT NULL,
    kco_id    TEXT NOT NULL,
    hash      TEXT NOT NULL,
    app_hash  TEXT NOT NULL,
    mtime     INTEGER NOT NULL,
    synced_at INTEGER NOT NULL
);
