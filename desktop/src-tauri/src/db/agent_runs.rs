use super::handle::{Db, new_id, now_ms, require_changed};
use super::settings;
use crate::agent::worktree::Worktree;
use crate::domain::agent::{
    AgentRun, AgentRunView, AgentSettings, RunAction, RunStatus, transition,
};
use crate::error::{AppError, AppResult};
use std::path::Path;

fn settings_key(project_id: &str) -> String {
    format!("project.{project_id}.agent")
}

pub async fn get_settings(db: &Db, project_id: &str) -> AppResult<AgentSettings> {
    match settings::get_string(db, &settings_key(project_id)).await? {
        Some(json) => Ok(serde_json::from_str(&json)?),
        None => Ok(AgentSettings::default()),
    }
}

pub async fn set_settings(
    db: &Db,
    project_id: &str,
    input: AgentSettings,
) -> AppResult<AgentSettings> {
    let clean = input.clean()?;
    settings::set_string(
        db,
        &settings_key(project_id),
        &serde_json::to_string(&clean)?,
    )
    .await?;
    Ok(clean)
}

/// A new run in the queue. Its log goes to `<log_dir>/<id>.jsonl`.
pub async fn create(
    db: &Db,
    project_id: &str,
    task_id: &str,
    log_dir: &Path,
) -> AppResult<AgentRun> {
    let id = new_id();
    let log_path = log_dir.join(format!("{id}.jsonl"));
    let now = now_ms();
    sqlx::query(
        "INSERT INTO agent_runs (id, created_at, updated_at, project_id, task_id, status, log_path)
         VALUES (?, ?, ?, ?, ?, 'queued', ?)",
    )
    .bind(&id)
    .bind(now)
    .bind(now)
    .bind(project_id)
    .bind(task_id)
    .bind(log_path.to_string_lossy().as_ref())
    .execute(&db.pool)
    .await?;
    get(db, &id).await
}

pub async fn get(db: &Db, id: &str) -> AppResult<AgentRun> {
    sqlx::query_as("SELECT * FROM agent_runs WHERE id = ?")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("run"))
}

/// Newest first, with the task title for the board.
pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<AgentRunView>> {
    let rows = sqlx::query_as(
        "SELECT r.*, COALESCE(t.title, '') AS task_title
           FROM agent_runs r LEFT JOIN tasks t ON t.id = r.task_id
          WHERE r.project_id = ? ORDER BY r.created_at DESC, r.id DESC",
    )
    .bind(project_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// The oldest queued run, the next one to start.
pub async fn next_queued(db: &Db) -> AppResult<Option<AgentRun>> {
    let run = sqlx::query_as(
        "SELECT * FROM agent_runs WHERE status = 'queued' ORDER BY created_at, id LIMIT 1",
    )
    .fetch_optional(&db.pool)
    .await?;
    Ok(run)
}

/// Moves the run along the status machine; `error` is kept for a failure.
pub async fn apply(
    db: &Db,
    id: &str,
    action: RunAction,
    error: Option<&str>,
) -> AppResult<AgentRun> {
    let run = get(db, id).await?;
    let status = transition(run.status, action)?;
    let now = now_ms();
    let result = sqlx::query(
        "UPDATE agent_runs
            SET status = ?, updated_at = ?, error = ?,
                started_at = CASE WHEN ? THEN ? ELSE started_at END,
                finished_at = CASE WHEN ? THEN ? ELSE finished_at END
          WHERE id = ? AND status = ?",
    )
    .bind(status)
    .bind(now)
    .bind(error)
    .bind(action == RunAction::Start)
    .bind(now)
    .bind(matches!(
        status,
        RunStatus::Review | RunStatus::Failed | RunStatus::Done | RunStatus::Discarded
    ))
    .bind(now)
    .bind(id)
    .bind(run.status)
    .execute(&db.pool)
    .await?;
    // another writer moved the run first
    require_changed(result, "run")?;
    get(db, id).await
}

pub async fn set_worktree(db: &Db, id: &str, worktree: &Worktree) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE agent_runs
            SET branch = ?, worktree_path = ?, base_branch = ?, base_commit = ?, updated_at = ?
          WHERE id = ?",
    )
    .bind(&worktree.branch)
    .bind(worktree.path.to_string_lossy().as_ref())
    .bind(&worktree.base_branch)
    .bind(&worktree.base_commit)
    .bind(now_ms())
    .bind(id)
    .execute(&db.pool)
    .await?;
    require_changed(result, "run")
}

pub async fn set_progress(
    db: &Db,
    id: &str,
    turns: i64,
    test_exit_code: Option<i64>,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE agent_runs SET turns = ?, test_exit_code = ?, updated_at = ? WHERE id = ?",
    )
    .bind(turns)
    .bind(test_exit_code)
    .bind(now_ms())
    .bind(id)
    .execute(&db.pool)
    .await?;
    require_changed(result, "run")
}

/// The commit made when the changes were approved.
pub async fn set_commit(db: &Db, id: &str, commit_sha: &str) -> AppResult<()> {
    let result = sqlx::query("UPDATE agent_runs SET commit_sha = ?, updated_at = ? WHERE id = ?")
        .bind(commit_sha)
        .bind(now_ms())
        .bind(id)
        .execute(&db.pool)
        .await?;
    require_changed(result, "run")
}

/// Queues the run again with what the user wants changed, or nothing for a plain retry.
pub async fn requeue(db: &Db, id: &str, feedback: &str) -> AppResult<AgentRun> {
    let run = apply(db, id, RunAction::Requeue, None).await?;
    sqlx::query(
        "UPDATE agent_runs
            SET feedback = ?, commit_sha = NULL, test_exit_code = NULL, turns = 0,
                finished_at = NULL, updated_at = ?
          WHERE id = ?",
    )
    .bind(feedback.trim())
    .bind(now_ms())
    .bind(&run.id)
    .execute(&db.pool)
    .await?;
    get(db, id).await
}

/// A run left `running` by an app that closed is failed on the next start.
pub async fn fail_interrupted(db: &Db) -> AppResult<u64> {
    let now = now_ms();
    let result = sqlx::query(
        "UPDATE agent_runs
            SET status = 'failed', error = 'the app was closed during the run',
                updated_at = ?, finished_at = ?
          WHERE status = 'running'",
    )
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{self, create_input};
    use std::path::PathBuf;

    async fn run_for(db: &Db) -> (String, AgentRun) {
        let project = create_for_test(db, "P").await;
        let task = tasks::create(db, create_input(&project.id, "Do it"))
            .await
            .expect("task");
        let run = create(db, &project.id, &task.id, &PathBuf::from("logs"))
            .await
            .expect("run");
        (project.id, run)
    }

    #[tokio::test]
    async fn a_new_run_is_queued_with_its_log_path() {
        let db = Db::in_memory().await;
        let (_, run) = run_for(&db).await;
        assert_eq!(run.status, RunStatus::Queued);
        assert!(run.log_path.ends_with(&format!("{}.jsonl", run.id)));
        assert_eq!(
            next_queued(&db).await.expect("next").expect("some").id,
            run.id
        );
    }

    #[tokio::test]
    async fn apply_follows_the_status_machine_and_stamps_times() {
        let db = Db::in_memory().await;
        let (_, run) = run_for(&db).await;

        let running = apply(&db, &run.id, RunAction::Start, None)
            .await
            .expect("start");
        assert_eq!(running.status, RunStatus::Running);
        assert!(running.started_at.is_some() && running.finished_at.is_none());
        assert!(next_queued(&db).await.expect("next").is_none());

        let failed = apply(&db, &run.id, RunAction::Fail, Some("boom"))
            .await
            .expect("fail");
        assert_eq!(failed.status, RunStatus::Failed);
        assert_eq!(failed.error.as_deref(), Some("boom"));
        assert!(failed.finished_at.is_some());

        let err = apply(&db, &run.id, RunAction::Finish, None).await;
        assert!(matches!(err, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn the_board_list_has_task_titles_newest_first() {
        let db = Db::in_memory().await;
        let (project_id, first) = run_for(&db).await;
        let task = tasks::create(&db, create_input(&project_id, "Second"))
            .await
            .expect("task");
        let second = create(&db, &project_id, &task.id, &PathBuf::from("logs"))
            .await
            .expect("run");

        let rows = list(&db, &project_id).await.expect("list");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].run.id, second.id);
        assert_eq!(rows[0].task_title, "Second");
        assert_eq!(rows[1].run.id, first.id);
    }

    #[tokio::test]
    async fn worktree_and_progress_are_stored() {
        let db = Db::in_memory().await;
        let (_, run) = run_for(&db).await;
        let worktree = Worktree {
            branch: "agent/do-it".into(),
            path: PathBuf::from("wt/1"),
            base_branch: Some("main".into()),
            base_commit: "abc".into(),
        };
        set_worktree(&db, &run.id, &worktree)
            .await
            .expect("worktree");
        set_progress(&db, &run.id, 7, Some(0))
            .await
            .expect("progress");

        let stored = get(&db, &run.id).await.expect("get");
        assert_eq!(stored.branch.as_deref(), Some("agent/do-it"));
        assert_eq!(stored.base_branch.as_deref(), Some("main"));
        assert_eq!((stored.turns, stored.test_exit_code), (7, Some(0)));
    }

    #[tokio::test]
    async fn only_running_runs_fail_on_start() {
        let db = Db::in_memory().await;
        let (_, run) = run_for(&db).await;
        assert_eq!(fail_interrupted(&db).await.expect("none"), 0);

        apply(&db, &run.id, RunAction::Start, None)
            .await
            .expect("start");
        assert_eq!(fail_interrupted(&db).await.expect("one"), 1);
        let stored = get(&db, &run.id).await.expect("get");
        assert_eq!(stored.status, RunStatus::Failed);
        assert!(stored.error.expect("error").contains("closed"));
    }

    #[tokio::test]
    async fn settings_default_then_round_trip_cleaned() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        assert_eq!(
            get_settings(&db, &project.id).await.expect("default"),
            AgentSettings::default()
        );

        let input = AgentSettings {
            test_command: "  npm test  ".into(),
            ..AgentSettings::default()
        };
        let saved = set_settings(&db, &project.id, input).await.expect("save");
        assert_eq!(saved.test_command, "npm test");
        assert_eq!(get_settings(&db, &project.id).await.expect("get"), saved);

        let bad = AgentSettings {
            timeout_min: 0,
            ..AgentSettings::default()
        };
        assert!(set_settings(&db, &project.id, bad).await.is_err());
    }
}
