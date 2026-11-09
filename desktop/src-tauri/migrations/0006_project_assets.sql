CREATE TABLE project_assets (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    device_id   TEXT NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0,
    project_id  TEXT NOT NULL REFERENCES projects (id),
    kind        TEXT NOT NULL CHECK (kind IN ('screenshot', 'link')),
    -- a screenshot is the file name inside the app's assets folder, a link is a url
    path_or_url TEXT NOT NULL,
    caption     TEXT NOT NULL DEFAULT '',
    order_key   TEXT NOT NULL
);

CREATE INDEX idx_project_assets_project ON project_assets (project_id, order_key);
