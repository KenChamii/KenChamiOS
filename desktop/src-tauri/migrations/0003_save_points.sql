CREATE TABLE save_points (
    id                 TEXT PRIMARY KEY NOT NULL,
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL,
    device_id          TEXT NOT NULL,
    deleted            INTEGER NOT NULL DEFAULT 0,
    project_id         TEXT NOT NULL REFERENCES projects (id),
    stage              TEXT NOT NULL
        CHECK (stage IN ('idea', 'plan', 'build', 'review', 'done')),
    source             TEXT NOT NULL DEFAULT 'manual'
        CHECK (source IN ('manual', 'session_note')),
    note               TEXT NOT NULL,
    done_md            TEXT NOT NULL DEFAULT '',
    next_md            TEXT NOT NULL DEFAULT '',
    session_summary_md TEXT NOT NULL DEFAULT '',
    git_branch         TEXT,
    git_commit         TEXT
);

CREATE INDEX idx_save_points_project ON save_points (project_id, created_at);
