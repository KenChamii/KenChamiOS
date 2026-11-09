use super::handle::Db;
use super::settings;
use crate::domain::idle::{
    DEFAULT_IDLE_DAYS, IdleProject, ProjectQuiet, idle_projects, validate_idle_days,
};
use crate::domain::project::Stage;
use crate::error::{AppError, AppResult};
use sqlx::FromRow;

const IDLE_DAYS_KEY: &str = "reminders.idle_days";
const SNOOZE_PREFIX: &str = "reminders.snooze.";
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(FromRow)]
struct QuietRow {
    project_id: String,
    name: String,
    stage: Stage,
    last_work_at: i64,
    last_save_point_at: Option<i64>,
    snoozed_until: Option<String>,
}

impl From<QuietRow> for ProjectQuiet {
    fn from(row: QuietRow) -> Self {
        Self {
            project_id: row.project_id,
            name: row.name,
            stage: row.stage,
            last_work_at: row.last_work_at,
            last_save_point_at: row.last_save_point_at,
            // settings hold json text; a value that does not parse means no snooze
            snoozed_until: row
                .snoozed_until
                .and_then(|json| serde_json::from_str::<String>(&json).ok())
                .and_then(|text| text.parse().ok()),
        }
    }
}

/// Days of quiet before a project is suggested; 7 until the user changes it.
pub async fn idle_days(db: &Db) -> AppResult<i64> {
    let stored = settings::get_string(db, IDLE_DAYS_KEY).await?;
    Ok(stored
        .and_then(|text| text.parse().ok())
        .filter(|days| validate_idle_days(*days).is_ok())
        .unwrap_or(DEFAULT_IDLE_DAYS))
}

pub async fn set_idle_days(db: &Db, days: i64) -> AppResult<i64> {
    validate_idle_days(days)?;
    settings::set_string(db, IDLE_DAYS_KEY, &days.to_string()).await?;
    Ok(days)
}

/// Active projects that went quiet without a save point, from one aggregate query.
pub async fn idle(db: &Db, now: i64) -> AppResult<Vec<IdleProject>> {
    let rows: Vec<QuietRow> = sqlx::query_as(
        "SELECT p.id AS project_id, p.name, p.stage,
                COALESCE((SELECT MAX(a.at) FROM activity a
                           WHERE a.deleted = 0
                             AND (a.entity_id = p.id
                              OR a.entity_id IN (SELECT id FROM tasks WHERE project_id = p.id)
                              OR a.entity_id IN (SELECT c.id FROM checklist_items c
                                                  JOIN tasks t ON t.id = c.task_id
                                                 WHERE t.project_id = p.id)
                              OR a.entity_id IN
                                 (SELECT id FROM project_docs WHERE project_id = p.id)
                              OR a.entity_id IN
                                 (SELECT id FROM time_entries WHERE project_id = p.id)
                              OR a.entity_id IN
                                 (SELECT id FROM project_assets WHERE project_id = p.id))),
                         p.created_at) AS last_work_at,
                (SELECT MAX(s.created_at) FROM save_points s
                  WHERE s.project_id = p.id AND s.deleted = 0) AS last_save_point_at,
                (SELECT value_json FROM settings
                  WHERE key = ?1 || p.id AND deleted = 0) AS snoozed_until
           FROM projects p
          WHERE p.deleted = 0 AND p.archived = 0 AND p.stage IN ('plan', 'build', 'review')",
    )
    .bind(SNOOZE_PREFIX)
    .fetch_all(&db.pool)
    .await?;
    let quiet: Vec<ProjectQuiet> = rows.into_iter().map(ProjectQuiet::from).collect();
    Ok(idle_projects(&quiet, now, idle_days(db).await?))
}

/// Hides the project from the suggestions for as long as the reminder delay.
pub async fn snooze(db: &Db, project_id: &str, now: i64) -> AppResult<()> {
    let known: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM projects WHERE id = ? AND deleted = 0)")
            .bind(project_id)
            .fetch_one(&db.pool)
            .await?;
    if !known {
        return Err(AppError::NotFound("project"));
    }
    let until = now + idle_days(db).await? * DAY_MS;
    let key = format!("{SNOOZE_PREFIX}{project_id}");
    settings::set_string(db, &key, &until.to_string()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{now_ms, projects, save_points, tasks};
    use crate::domain::SavePointInput;

    async fn age_activity(db: &Db, days: i64) {
        sqlx::query("UPDATE activity SET at = at - ?")
            .bind(days * DAY_MS)
            .execute(&db.pool)
            .await
            .expect("age activity");
        sqlx::query("UPDATE projects SET created_at = created_at - ?")
            .bind(days * DAY_MS)
            .execute(&db.pool)
            .await
            .expect("age projects");
    }

    async fn building(db: &Db, name: &str) -> String {
        let project = projects::create_for_test(db, name).await;
        projects::set_stage(db, &project.id, Stage::Build)
            .await
            .expect("stage");
        project.id
    }

    #[tokio::test]
    async fn the_delay_defaults_to_seven_days_and_is_validated() {
        let db = Db::in_memory().await;
        assert_eq!(idle_days(&db).await.expect("default"), 7);

        assert_eq!(set_idle_days(&db, 14).await.expect("set"), 14);
        assert_eq!(idle_days(&db).await.expect("read"), 14);
        assert!(set_idle_days(&db, 0).await.is_err());
        assert_eq!(idle_days(&db).await.expect("unchanged"), 14);
    }

    #[tokio::test]
    async fn a_garbled_stored_delay_falls_back_to_the_default() {
        let db = Db::in_memory().await;
        settings::set_string(&db, IDLE_DAYS_KEY, "soon")
            .await
            .expect("set");

        assert_eq!(idle_days(&db).await.expect("read"), 7);
    }

    #[tokio::test]
    async fn a_project_with_recent_work_is_not_idle() {
        let db = Db::in_memory().await;
        building(&db, "Fresh").await;

        assert!(idle(&db, now_ms()).await.expect("idle").is_empty());
    }

    #[tokio::test]
    async fn quiet_work_without_a_save_point_is_suggested() {
        let db = Db::in_memory().await;
        let id = building(&db, "Quiet").await;
        age_activity(&db, 10).await;

        let found = idle(&db, now_ms()).await.expect("idle");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].project_id, id);
        assert_eq!(found[0].idle_days, 10);
        assert_eq!(found[0].last_save_point_at, None);
    }

    #[tokio::test]
    async fn a_save_point_made_after_the_work_clears_the_suggestion() {
        let db = Db::in_memory().await;
        let id = building(&db, "Saved").await;
        age_activity(&db, 10).await;
        let input = SavePointInput {
            note: "Stopped here".to_string(),
            done_md: String::new(),
            next_md: String::new(),
        };
        save_points::create(&db, &id, input, None, None)
            .await
            .expect("save point");
        sqlx::query("UPDATE save_points SET created_at = created_at - ?")
            .bind(9 * DAY_MS)
            .execute(&db.pool)
            .await
            .expect("age save point");

        assert!(idle(&db, now_ms()).await.expect("idle").is_empty());
    }

    #[tokio::test]
    async fn work_on_a_task_counts_as_activity() {
        let db = Db::in_memory().await;
        let id = building(&db, "Busy").await;
        age_activity(&db, 10).await;
        tasks::create(&db, tasks::create_input(&id, "Do it"))
            .await
            .expect("task");

        assert!(idle(&db, now_ms()).await.expect("idle").is_empty());
    }

    #[tokio::test]
    async fn snoozing_hides_a_project_for_the_delay_and_unknown_projects_fail() {
        let db = Db::in_memory().await;
        let id = building(&db, "Snoozed").await;
        age_activity(&db, 10).await;
        let start = now_ms();

        snooze(&db, &id, start).await.expect("snooze");

        assert!(idle(&db, start).await.expect("hidden").is_empty());
        let after = start + 7 * DAY_MS;
        assert_eq!(idle(&db, after).await.expect("again").len(), 1);
        assert!(matches!(
            snooze(&db, "nope", start).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn archived_and_early_stage_projects_are_ignored() {
        let db = Db::in_memory().await;
        projects::create_for_test(&db, "Idea").await;
        let archived = building(&db, "Archived").await;
        sqlx::query("UPDATE projects SET archived = 1 WHERE id = ?")
            .bind(archived)
            .execute(&db.pool)
            .await
            .expect("archive");
        age_activity(&db, 30).await;

        assert!(idle(&db, now_ms()).await.expect("idle").is_empty());
    }
}
