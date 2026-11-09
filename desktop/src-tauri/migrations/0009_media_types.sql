-- sqlite cannot change a check constraint, so the three media tables are rebuilt, children first
CREATE TABLE old_items AS SELECT * FROM media_items;
CREATE TABLE old_entries AS SELECT * FROM media_entries;
CREATE TABLE old_log AS SELECT * FROM media_progress_log;

DROP TABLE media_progress_log;
DROP TABLE media_entries;
DROP TABLE media_items;

CREATE TABLE media_items (
    id               TEXT PRIMARY KEY NOT NULL,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,
    device_id        TEXT NOT NULL,
    deleted          INTEGER NOT NULL DEFAULT 0,
    type             TEXT NOT NULL CHECK (type IN ('anime', 'manga', 'game', 'movie', 'show', 'album', 'artist')),
    title            TEXT NOT NULL,
    cover_url        TEXT,
    -- file name inside the app's covers folder; each device caches its own
    cover_cache_path TEXT,
    year             INTEGER,
    genres           TEXT NOT NULL DEFAULT '[]',
    synopsis         TEXT NOT NULL DEFAULT '',
    source           TEXT NOT NULL CHECK (source IN ('anilist', 'rawg', 'tmdb', 'musicbrainz', 'manual')),
    external_id      TEXT
);

-- the same title from the same source is one item
CREATE UNIQUE INDEX idx_media_items_external
    ON media_items (source, external_id) WHERE external_id IS NOT NULL AND deleted = 0;

CREATE TABLE media_entries (
    id             TEXT PRIMARY KEY NOT NULL,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL,
    device_id      TEXT NOT NULL,
    deleted        INTEGER NOT NULL DEFAULT 0,
    media_item_id  TEXT NOT NULL REFERENCES media_items (id),
    status         TEXT NOT NULL
        CHECK (status IN ('in_progress', 'planned', 'completed', 'on_hold', 'dropped')),
    -- episodes for anime, chapters for manga, hours for games
    progress       INTEGER NOT NULL DEFAULT 0 CHECK (progress >= 0),
    progress_total INTEGER CHECK (progress_total >= 0),
    rating         INTEGER CHECK (rating BETWEEN 1 AND 10),
    notes          TEXT NOT NULL DEFAULT '',
    tags           TEXT NOT NULL DEFAULT '[]',
    started_at     INTEGER,
    finished_at    INTEGER,
    platform       TEXT
);

-- one library entry per item
CREATE UNIQUE INDEX idx_media_entries_item ON media_entries (media_item_id) WHERE deleted = 0;

CREATE TABLE media_progress_log (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    device_id   TEXT NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0,
    entry_id    TEXT NOT NULL REFERENCES media_entries (id),
    delta       INTEGER NOT NULL,
    value_after INTEGER NOT NULL,
    logged_at   INTEGER NOT NULL
);

CREATE INDEX idx_media_progress_log_entry ON media_progress_log (entry_id, logged_at);

INSERT INTO media_items SELECT * FROM old_items;
INSERT INTO media_entries SELECT * FROM old_entries;
INSERT INTO media_progress_log SELECT * FROM old_log;

DROP TABLE old_log;
DROP TABLE old_entries;
DROP TABLE old_items;
