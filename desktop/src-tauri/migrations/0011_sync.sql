-- one entry per changed row; triggers cover every write path, so no repository can forget to log
CREATE TABLE change_log (
    seq        INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    table_name TEXT NOT NULL,
    row_id     TEXT NOT NULL,
    hlc        INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_change_log_row ON change_log (table_name, row_id);

-- the phones allowed to sync; only a hash of their token is kept
CREATE TABLE paired_devices (
    device_id    TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    token_hash   TEXT NOT NULL,
    paired_at    INTEGER NOT NULL,
    last_sync_at INTEGER,
    pulled_seq   INTEGER NOT NULL DEFAULT 0
);

CREATE TRIGGER trg_activity_log_insert AFTER INSERT ON activity
BEGIN
    DELETE FROM change_log WHERE table_name = 'activity' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('activity', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_activity_log_update AFTER UPDATE ON activity
BEGIN
    DELETE FROM change_log WHERE table_name = 'activity' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('activity', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_checklist_items_log_insert AFTER INSERT ON checklist_items
BEGIN
    DELETE FROM change_log WHERE table_name = 'checklist_items' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('checklist_items', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_checklist_items_log_update AFTER UPDATE ON checklist_items
BEGIN
    DELETE FROM change_log WHERE table_name = 'checklist_items' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('checklist_items', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_media_entries_log_insert AFTER INSERT ON media_entries
BEGIN
    DELETE FROM change_log WHERE table_name = 'media_entries' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('media_entries', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_media_entries_log_update AFTER UPDATE ON media_entries
BEGIN
    DELETE FROM change_log WHERE table_name = 'media_entries' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('media_entries', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_media_items_log_insert AFTER INSERT ON media_items
BEGIN
    DELETE FROM change_log WHERE table_name = 'media_items' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('media_items', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_media_items_log_update AFTER UPDATE ON media_items
BEGIN
    DELETE FROM change_log WHERE table_name = 'media_items' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('media_items', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_media_progress_log_log_insert AFTER INSERT ON media_progress_log
BEGIN
    DELETE FROM change_log WHERE table_name = 'media_progress_log' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('media_progress_log', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_media_progress_log_log_update AFTER UPDATE ON media_progress_log
BEGIN
    DELETE FROM change_log WHERE table_name = 'media_progress_log' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('media_progress_log', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_project_docs_log_insert AFTER INSERT ON project_docs
BEGIN
    DELETE FROM change_log WHERE table_name = 'project_docs' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('project_docs', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_project_docs_log_update AFTER UPDATE ON project_docs
BEGIN
    DELETE FROM change_log WHERE table_name = 'project_docs' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('project_docs', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_projects_log_insert AFTER INSERT ON projects
BEGIN
    DELETE FROM change_log WHERE table_name = 'projects' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('projects', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_projects_log_update AFTER UPDATE ON projects
BEGIN
    DELETE FROM change_log WHERE table_name = 'projects' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('projects', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_prompts_log_insert AFTER INSERT ON prompts
BEGIN
    DELETE FROM change_log WHERE table_name = 'prompts' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('prompts', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_prompts_log_update AFTER UPDATE ON prompts
BEGIN
    DELETE FROM change_log WHERE table_name = 'prompts' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('prompts', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_save_points_log_insert AFTER INSERT ON save_points
BEGIN
    DELETE FROM change_log WHERE table_name = 'save_points' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('save_points', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_save_points_log_update AFTER UPDATE ON save_points
BEGIN
    DELETE FROM change_log WHERE table_name = 'save_points' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('save_points', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_tasks_log_insert AFTER INSERT ON tasks
BEGIN
    DELETE FROM change_log WHERE table_name = 'tasks' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('tasks', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_tasks_log_update AFTER UPDATE ON tasks
BEGIN
    DELETE FROM change_log WHERE table_name = 'tasks' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('tasks', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_time_entries_log_insert AFTER INSERT ON time_entries
BEGIN
    DELETE FROM change_log WHERE table_name = 'time_entries' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('time_entries', NEW.id, NEW.updated_at);
END;

CREATE TRIGGER trg_time_entries_log_update AFTER UPDATE ON time_entries
BEGIN
    DELETE FROM change_log WHERE table_name = 'time_entries' AND row_id = NEW.id;
    INSERT INTO change_log (table_name, row_id, hlc) VALUES ('time_entries', NEW.id, NEW.updated_at);
END;

-- rows that exist before this migration go out with the first sync
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'activity', id, updated_at FROM activity;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'checklist_items', id, updated_at FROM checklist_items;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'media_entries', id, updated_at FROM media_entries;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'media_items', id, updated_at FROM media_items;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'media_progress_log', id, updated_at FROM media_progress_log;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'project_docs', id, updated_at FROM project_docs;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'projects', id, updated_at FROM projects;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'prompts', id, updated_at FROM prompts;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'save_points', id, updated_at FROM save_points;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'tasks', id, updated_at FROM tasks;
INSERT INTO change_log (table_name, row_id, hlc) SELECT 'time_entries', id, updated_at FROM time_entries;
