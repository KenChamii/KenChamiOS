-- desktop-only, never synced: a run points at local folders, log files and a process.
-- no foreign keys, so the sync purge of old tombstoned tasks is never blocked by a run
CREATE TABLE agent_runs (
    id             TEXT PRIMARY KEY NOT NULL,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL,
    project_id     TEXT NOT NULL,
    task_id        TEXT NOT NULL,
    status         TEXT NOT NULL
        CHECK (status IN ('queued', 'running', 'review', 'done', 'failed', 'discarded')),
    branch         TEXT,
    worktree_path  TEXT,
    base_branch    TEXT,
    base_commit    TEXT,
    -- the commit made on approve
    commit_sha     TEXT,
    feedback       TEXT NOT NULL DEFAULT '',
    error          TEXT,
    turns          INTEGER NOT NULL DEFAULT 0,
    test_exit_code INTEGER,
    log_path       TEXT NOT NULL,
    started_at     INTEGER,
    finished_at    INTEGER
);

CREATE INDEX idx_agent_runs_project ON agent_runs (project_id, created_at);
