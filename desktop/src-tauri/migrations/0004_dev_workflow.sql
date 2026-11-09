CREATE TABLE export_templates (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    kind       TEXT NOT NULL CHECK (kind IN ('brief', 'task_prompt', 'portfolio')),
    name       TEXT NOT NULL,
    body       TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0
);

-- one active template per kind for now
CREATE UNIQUE INDEX idx_export_templates_default ON export_templates (kind)
    WHERE is_default = 1 AND deleted = 0;

CREATE TABLE prompts (
    id           TEXT PRIMARY KEY NOT NULL,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL,
    device_id    TEXT NOT NULL,
    deleted      INTEGER NOT NULL DEFAULT 0,
    project_id   TEXT REFERENCES projects (id),
    title        TEXT NOT NULL,
    body_md      TEXT NOT NULL DEFAULT '',
    tags         TEXT NOT NULL DEFAULT '[]',
    favorite     INTEGER NOT NULL DEFAULT 0,
    use_count    INTEGER NOT NULL DEFAULT 0,
    last_used_at INTEGER
);

CREATE INDEX idx_prompts_project ON prompts (project_id);
