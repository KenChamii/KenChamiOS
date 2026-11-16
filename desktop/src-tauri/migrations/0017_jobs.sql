-- desktop-only for now: the Android screen is deferred, so the table is not synced yet.
-- it still carries the sync columns, so turning sync on later needs no data change
CREATE TABLE job_applications (
    id               TEXT PRIMARY KEY NOT NULL,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,
    device_id        TEXT NOT NULL,
    deleted          INTEGER NOT NULL DEFAULT 0,
    company          TEXT NOT NULL,
    role             TEXT NOT NULL,
    url              TEXT NOT NULL DEFAULT '',
    location         TEXT NOT NULL DEFAULT '',
    salary           TEXT NOT NULL DEFAULT '',
    source           TEXT NOT NULL DEFAULT 'other'
        CHECK (source IN ('linkedin', 'company_site', 'job_board', 'referral', 'recruiter', 'other')),
    stage            TEXT NOT NULL DEFAULT 'saved'
        CHECK (stage IN ('saved', 'applied', 'screening', 'interview', 'offer', 'accepted', 'rejected', 'withdrawn')),
    -- local day, YYYY-MM-DD
    applied_on       TEXT,
    contact          TEXT NOT NULL DEFAULT '',
    resume_version   TEXT NOT NULL DEFAULT '',
    next_step        TEXT NOT NULL DEFAULT '',
    next_step_at     INTEGER,
    notes_md         TEXT NOT NULL DEFAULT '',
    stage_changed_at INTEGER NOT NULL
);

CREATE INDEX idx_job_applications_stage ON job_applications (stage) WHERE deleted = 0;

-- when each application entered each stage, for the time-per-stage numbers
CREATE TABLE job_stage_log (
    job_id     TEXT NOT NULL,
    stage      TEXT NOT NULL,
    entered_at INTEGER NOT NULL
);

CREATE INDEX idx_job_stage_log_job ON job_stage_log (job_id, entered_at);
