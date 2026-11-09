-- answers of the metadata providers, kept on this device only and never synced
CREATE TABLE metadata_cache (
    cache_key  TEXT PRIMARY KEY NOT NULL,
    payload    TEXT NOT NULL,
    fetched_at INTEGER NOT NULL
);
