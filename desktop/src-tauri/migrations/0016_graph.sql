-- desktop-only, never synced: an index of the notes in this device's vault, and the links
-- between notes, projects, days and tags that the graph draws. Rebuilt from the vault and the app.
CREATE TABLE vault_notes (
    id       TEXT PRIMARY KEY NOT NULL,
    path     TEXT NOT NULL,
    title    TEXT NOT NULL,
    tags     TEXT NOT NULL DEFAULT '[]',
    headings TEXT NOT NULL DEFAULT '[]',
    mtime    INTEGER NOT NULL
);

CREATE TABLE links (
    source_id   TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    target_id   TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    kind        TEXT NOT NULL,
    PRIMARY KEY (source_id, target_id, kind)
);

CREATE INDEX idx_links_target ON links (target_id);
