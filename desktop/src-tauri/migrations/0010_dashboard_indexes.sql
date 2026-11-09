-- the dashboard filters by time range and due date across all projects
CREATE INDEX idx_time_entries_started ON time_entries (started_at) WHERE deleted = 0;

CREATE INDEX idx_media_progress_log_logged ON media_progress_log (logged_at) WHERE deleted = 0;

CREATE INDEX idx_media_entries_finished ON media_entries (finished_at)
    WHERE deleted = 0 AND finished_at IS NOT NULL;

CREATE INDEX idx_tasks_due ON tasks (due_at)
    WHERE deleted = 0 AND due_at IS NOT NULL AND status != 'done';
