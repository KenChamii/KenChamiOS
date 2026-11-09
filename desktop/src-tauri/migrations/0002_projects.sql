CREATE TABLE projects (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    device_id   TEXT NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL,
    stage       TEXT NOT NULL DEFAULT 'idea'
        CHECK (stage IN ('idea', 'plan', 'build', 'review', 'done')),
    description TEXT NOT NULL DEFAULT '',
    repo_path   TEXT,
    color       TEXT,
    tags        TEXT NOT NULL DEFAULT '[]',
    archived    INTEGER NOT NULL DEFAULT 0,
    finished_at INTEGER
);

CREATE TABLE project_docs (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    project_id TEXT NOT NULL REFERENCES projects (id),
    kind       TEXT NOT NULL CHECK (kind IN ('idea', 'plan', 'adr', 'note')),
    title      TEXT NOT NULL DEFAULT '',
    body_md    TEXT NOT NULL DEFAULT ''
);

-- one doc per kind and project for now
CREATE UNIQUE INDEX idx_project_docs_kind ON project_docs (project_id, kind) WHERE deleted = 0;

CREATE TABLE tasks (
    id            TEXT PRIMARY KEY NOT NULL,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    device_id     TEXT NOT NULL,
    deleted       INTEGER NOT NULL DEFAULT 0,
    project_id    TEXT NOT NULL REFERENCES projects (id),
    parent_id     TEXT REFERENCES tasks (id),
    title         TEXT NOT NULL,
    body_md       TEXT NOT NULL DEFAULT '',
    acceptance_md TEXT NOT NULL DEFAULT '',
    related_files TEXT NOT NULL DEFAULT '[]',
    status        TEXT NOT NULL DEFAULT 'todo'
        CHECK (status IN ('todo', 'doing', 'review', 'done')),
    priority      TEXT NOT NULL DEFAULT 'medium'
        CHECK (priority IN ('low', 'medium', 'high')),
    due_at        INTEGER,
    order_key     TEXT NOT NULL,
    tags          TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX idx_tasks_project ON tasks (project_id, status, order_key);

CREATE TABLE checklist_items (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    task_id    TEXT NOT NULL REFERENCES tasks (id),
    text       TEXT NOT NULL,
    done       INTEGER NOT NULL DEFAULT 0,
    order_key  TEXT NOT NULL
);

CREATE INDEX idx_checklist_task ON checklist_items (task_id, order_key);
