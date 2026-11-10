-- e.g. 'weekly:2'; null for a one-off task
ALTER TABLE tasks ADD COLUMN recurrence TEXT;

CREATE TABLE habits (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    name       TEXT NOT NULL
);

-- one row per habit and local day ('YYYY-MM-DD')
CREATE TABLE habit_checks (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    device_id  TEXT NOT NULL,
    deleted    INTEGER NOT NULL DEFAULT 0,
    habit_id   TEXT NOT NULL REFERENCES habits (id),
    day        TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_habit_checks_day ON habit_checks (habit_id, day) WHERE deleted = 0;

CREATE TRIGGER trg_habits_log_insert AFTER INSERT ON habits
BEGIN
    DELETE FROM change_log WHERE table_name = 'habits' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('habits', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_habits_log_update AFTER UPDATE ON habits
BEGIN
    DELETE FROM change_log WHERE table_name = 'habits' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('habits', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_habit_checks_log_insert AFTER INSERT ON habit_checks
BEGIN
    DELETE FROM change_log WHERE table_name = 'habit_checks' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('habit_checks', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_habit_checks_log_update AFTER UPDATE ON habit_checks
BEGIN
    DELETE FROM change_log WHERE table_name = 'habit_checks' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('habit_checks', NEW.id, NEW.updated_at);
END;
