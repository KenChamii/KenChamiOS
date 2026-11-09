CREATE TABLE time_entries (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    project_id TEXT NOT NULL REFERENCES projects (id),
    task_id    TEXT REFERENCES tasks (id),
    kind       TEXT NOT NULL CHECK (kind IN ('focus', 'break', 'manual')),
    started_at INTEGER NOT NULL,
    ended_at   INTEGER NOT NULL,
    duration_s INTEGER NOT NULL CHECK (duration_s > 0),
    note       TEXT NOT NULL DEFAULT ''
);

CREATE INDEX idx_time_entries_project ON time_entries (project_id, started_at);

-- one row per device, its id is the device id; the timer is not synced
CREATE TABLE timer_state (
    id               TEXT PRIMARY KEY NOT NULL,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,
    device_id        TEXT NOT NULL,
    deleted          INTEGER NOT NULL DEFAULT 0,
    project_id       TEXT NOT NULL REFERENCES projects (id),
    task_id          TEXT NOT NULL REFERENCES tasks (id),
    phase            TEXT NOT NULL CHECK (phase IN ('focus', 'break', 'long_break')),
    status           TEXT NOT NULL CHECK (status IN ('running', 'paused')),
    round            INTEGER NOT NULL,
    phase_started_at INTEGER NOT NULL,
    phase_length_s   INTEGER NOT NULL,
    elapsed_ms       INTEGER NOT NULL,
    running_since    INTEGER
);
