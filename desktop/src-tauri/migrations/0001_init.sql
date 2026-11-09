CREATE TABLE settings (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    key        TEXT NOT NULL UNIQUE,
    value_json TEXT NOT NULL
);

CREATE TABLE activity (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    device_id   TEXT NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0,
    kind        TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    summary     TEXT NOT NULL,
    at          INTEGER NOT NULL
);

CREATE INDEX idx_activity_at ON activity (at);
