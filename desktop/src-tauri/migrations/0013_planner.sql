-- local day the task is planned for ('YYYY-MM-DD'); null when not planned
ALTER TABLE tasks ADD COLUMN planned_on TEXT;
-- start of the time block in unix ms; always comes with planned_on
ALTER TABLE tasks ADD COLUMN scheduled_at INTEGER;
-- estimate or block length in minutes, 1 to 1440
ALTER TABLE tasks ADD COLUMN duration_min INTEGER;

CREATE INDEX idx_tasks_planned_on ON tasks (planned_on) WHERE deleted = 0;

-- one markdown note per local day ('YYYY-MM-DD')
CREATE TABLE daily_notes (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    day        TEXT NOT NULL,
    body       TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_daily_notes_day ON daily_notes (day) WHERE deleted = 0;

CREATE TRIGGER trg_daily_notes_log_insert AFTER INSERT ON daily_notes
BEGIN
    DELETE FROM change_log WHERE table_name = 'daily_notes' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('daily_notes', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_daily_notes_log_update AFTER UPDATE ON daily_notes
BEGIN
    DELETE FROM change_log WHERE table_name = 'daily_notes' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('daily_notes', NEW.id, NEW.updated_at);
END;
