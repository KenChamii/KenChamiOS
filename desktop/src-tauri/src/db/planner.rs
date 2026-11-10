use super::handle::Db;
use super::{activity, settings, tasks};
use crate::domain::Task;
use crate::domain::civil_date;
use crate::domain::planner::{
    DEFAULT_AVAILABLE_MIN, FocusBlock, PlannedTask, TaskPlan, validate_available_minutes,
};
use crate::domain::project::INBOX_ID;
use crate::domain::search::like_pattern;
use crate::error::{AppError, AppResult};
use sqlx::AssertSqlSafe;

const AVAILABLE_KEY: &str = "planner.available_minutes";
const MAX_CANDIDATES: i64 = 50;

const PLANNED_SELECT: &str = "SELECT t.id, t.title, t.project_id, p.name AS project_name,
        p.color AS project_color, t.status, t.priority, t.due_at, t.recurrence, t.planned_on,
        t.scheduled_at, t.duration_min
   FROM tasks t JOIN projects p ON p.id = t.project_id
  WHERE t.deleted = 0 AND p.deleted = 0 AND p.archived = 0";

/// Replaces the task's plan; an empty plan takes it off the planner.
pub async fn set_plan(db: &Db, id: &str, plan: TaskPlan) -> AppResult<Task> {
    let plan = plan.clean()?;
    let mut tx = db.pool.begin().await?;
    let title: String = sqlx::query_scalar("SELECT title FROM tasks WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("task"))?;
    sqlx::query(
        "UPDATE tasks SET planned_on = ?, scheduled_at = ?, duration_min = ?, updated_at = ?,
                device_id = ?
          WHERE id = ?",
    )
    .bind(&plan.planned_on)
    .bind(plan.scheduled_at)
    .bind(plan.duration_min)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    let (kind, summary) = match (&plan.planned_on, plan.scheduled_at) {
        (None, _) => ("task.unplanned", format!("Unplanned {title}")),
        (Some(day), None) => ("task.planned", format!("Planned {title} for {day}")),
        (Some(day), Some(_)) => ("task.planned", format!("Scheduled {title} on {day}")),
    };
    activity::record(&mut tx, db, kind, "task", id, &summary).await?;
    tx.commit().await?;
    tasks::get(db, id).await
}

/// Tasks planned from `from` to `to` (local days, both included), scheduled ones by start time.
pub async fn planned(db: &Db, from: &str, to: &str) -> AppResult<Vec<PlannedTask>> {
    let (from, to) = day_range(from, to)?;
    let sql = format!(
        "{PLANNED_SELECT} AND t.planned_on >= ? AND t.planned_on <= ?
          ORDER BY t.planned_on, t.scheduled_at IS NULL, t.scheduled_at, t.order_key"
    );
    let rows = sqlx::query_as(AssertSqlSafe(sql))
        .bind(from)
        .bind(to)
        .fetch_all(&db.pool)
        .await?;
    Ok(rows)
}

/// Open tasks worth planning for `today`: due before `until`, in the Inbox, in progress or
/// left from an earlier day. With a `query`, open tasks whose title matches it instead.
pub async fn candidates(
    db: &Db,
    today: &str,
    until: i64,
    query: Option<&str>,
) -> AppResult<Vec<PlannedTask>> {
    let today = civil_date::clean(today)?;
    let pattern = query
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(like_pattern);
    let sql = format!(
        r"{PLANNED_SELECT} AND t.status != 'done'
            AND (t.planned_on IS NULL OR t.planned_on < ?1)
            AND CASE WHEN ?3 IS NULL
                     THEN t.due_at < ?2 OR t.project_id = ?4 OR t.status IN ('doing', 'review')
                          OR t.planned_on < ?1
                     ELSE t.title LIKE ?3 ESCAPE '\' END
          ORDER BY t.due_at IS NULL, t.due_at, t.created_at, t.id
          LIMIT ?5"
    );
    let rows = sqlx::query_as(AssertSqlSafe(sql))
        .bind(&today)
        .bind(until)
        .bind(pattern)
        .bind(INBOX_ID)
        .bind(MAX_CANDIDATES)
        .fetch_all(&db.pool)
        .await?;
    Ok(rows)
}

/// Done tasks that were completed in `[from, to)` (unix ms).
pub async fn completed(db: &Db, from: i64, to: i64) -> AppResult<Vec<PlannedTask>> {
    ms_range(from, to)?;
    let sql = format!(
        "{PLANNED_SELECT} AND t.status = 'done'
            AND EXISTS (SELECT 1 FROM activity a
                         WHERE a.entity_id = t.id AND a.kind = 'task.moved' AND a.deleted = 0
                           AND a.at >= ? AND a.at < ?)
          ORDER BY p.name, t.title"
    );
    let rows = sqlx::query_as(AssertSqlSafe(sql))
        .bind(from)
        .bind(to)
        .fetch_all(&db.pool)
        .await?;
    Ok(rows)
}

/// Focus and manual sessions that overlap `[from, to)`; breaks are left out.
pub async fn focus_blocks(db: &Db, from: i64, to: i64) -> AppResult<Vec<FocusBlock>> {
    ms_range(from, to)?;
    let rows = sqlx::query_as(
        "SELECT e.id, e.project_id, p.name AS project_name, p.color AS project_color,
                k.title AS task_title, e.started_at, e.ended_at
           FROM time_entries e
           JOIN projects p ON p.id = e.project_id
           LEFT JOIN tasks k ON k.id = e.task_id AND k.deleted = 0
          WHERE e.deleted = 0 AND e.kind != 'break' AND p.deleted = 0
            AND e.started_at < ? AND e.ended_at > ?
          ORDER BY e.started_at",
    )
    .bind(to)
    .bind(from)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// Minutes of planned work that fit in a day; 6 hours until the user changes it.
pub async fn available_minutes(db: &Db) -> AppResult<i64> {
    let stored = settings::get_string(db, AVAILABLE_KEY).await?;
    Ok(stored
        .and_then(|text| text.parse().ok())
        .filter(|minutes| validate_available_minutes(*minutes).is_ok())
        .unwrap_or(DEFAULT_AVAILABLE_MIN))
}

pub async fn set_available_minutes(db: &Db, minutes: i64) -> AppResult<i64> {
    validate_available_minutes(minutes)?;
    settings::set_string(db, AVAILABLE_KEY, &minutes.to_string()).await?;
    Ok(minutes)
}

fn day_range(from: &str, to: &str) -> AppResult<(String, String)> {
    let (from, to) = (civil_date::clean(from)?, civil_date::clean(to)?);
    if from > to {
        return Err(AppError::Invalid("the period must end after it starts"));
    }
    Ok((from, to))
}

fn ms_range(from: i64, to: i64) -> AppResult<()> {
    if from >= to {
        return Err(AppError::Invalid("the period must end after it starts"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{create, create_input, move_to};
    use crate::domain::TaskStatus;

    fn plan(day: Option<&str>, at: Option<i64>, minutes: Option<i64>) -> TaskPlan {
        TaskPlan {
            planned_on: day.map(str::to_string),
            scheduled_at: at,
            duration_min: minutes,
        }
    }

    fn ids(tasks: &[PlannedTask]) -> Vec<&str> {
        tasks.iter().map(|t| t.title.as_str()).collect()
    }

    #[tokio::test]
    async fn planning_scheduling_and_unplanning_a_task() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let task = create(&db, create_input(&project.id, "Write"))
            .await
            .expect("task");

        let planned = set_plan(&db, &task.id, plan(Some("2026-11-10"), None, Some(45)))
            .await
            .expect("plan");
        assert_eq!(planned.planned_on.as_deref(), Some("2026-11-10"));
        assert_eq!(planned.duration_min, Some(45));

        let scheduled = set_plan(
            &db,
            &task.id,
            plan(Some("2026-11-11"), Some(5_000), Some(90)),
        )
        .await
        .expect("schedule");
        assert_eq!(scheduled.scheduled_at, Some(5_000));

        let dropped = set_plan(&db, &task.id, TaskPlan::default())
            .await
            .expect("unplan");
        assert_eq!(
            (
                dropped.planned_on,
                dropped.scheduled_at,
                dropped.duration_min
            ),
            (None, None, None)
        );
        assert_eq!(
            kinds_for(&db, &task.id).await,
            [
                "task.created",
                "task.planned",
                "task.planned",
                "task.unplanned"
            ]
        );
    }

    #[tokio::test]
    async fn bad_plans_and_missing_tasks_are_refused() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let task = create(&db, create_input(&project.id, "Write"))
            .await
            .expect("task");
        for bad in [
            plan(Some("2026-13-01"), None, None),
            plan(None, Some(5_000), Some(30)),
            plan(Some("2026-11-10"), None, Some(0)),
            plan(Some("2026-11-10"), None, Some(1441)),
        ] {
            assert!(matches!(
                set_plan(&db, &task.id, bad).await,
                Err(AppError::Invalid(_))
            ));
        }
        assert!(matches!(
            set_plan(&db, "missing", TaskPlan::default()).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn planned_lists_the_days_in_range_with_blocks_first() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        for (title, day, at) in [
            ("Loose", "2026-11-10", None),
            ("Late", "2026-11-10", Some(9_000)),
            ("Early", "2026-11-10", Some(1_000)),
            ("Next week", "2026-11-17", None),
        ] {
            let task = create(&db, create_input(&project.id, title))
                .await
                .expect("task");
            set_plan(&db, &task.id, plan(Some(day), at, Some(30)))
                .await
                .expect("plan");
        }

        let week = planned(&db, "2026-11-09", "2026-11-15")
            .await
            .expect("week");

        assert_eq!(ids(&week), ["Early", "Late", "Loose"]);
        assert!(planned(&db, "2026-11-15", "2026-11-09").await.is_err());
    }

    #[tokio::test]
    async fn candidates_suggest_due_inbox_doing_and_leftover_tasks() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let mut due = create_input(&project.id, "Due");
        due.due_at = Some(100);
        create(&db, due).await.expect("due");
        let mut later = create_input(&project.id, "Due later");
        later.due_at = Some(10_000);
        create(&db, later).await.expect("later");
        create(&db, create_input(INBOX_ID, "Captured"))
            .await
            .expect("inbox");
        let doing = create(&db, create_input(&project.id, "Doing"))
            .await
            .expect("doing");
        move_to(&db, &doing.id, TaskStatus::Doing, None, None)
            .await
            .expect("move");
        let leftover = create(&db, create_input(&project.id, "Leftover"))
            .await
            .expect("leftover");
        set_plan(&db, &leftover.id, plan(Some("2026-11-09"), None, None))
            .await
            .expect("plan");
        let today = create(&db, create_input(&project.id, "Already today"))
            .await
            .expect("today");
        set_plan(&db, &today.id, plan(Some("2026-11-10"), None, None))
            .await
            .expect("plan");
        create(&db, create_input(&project.id, "Someday"))
            .await
            .expect("someday");

        let suggested = candidates(&db, "2026-11-10", 1_000, None)
            .await
            .expect("suggested");
        assert_eq!(ids(&suggested), ["Due", "Captured", "Doing", "Leftover"]);

        let found = candidates(&db, "2026-11-10", 1_000, Some("some"))
            .await
            .expect("search");
        assert_eq!(ids(&found), ["Someday"]);
    }

    #[tokio::test]
    async fn completed_reads_the_completion_time() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let done = create(&db, create_input(&project.id, "Done"))
            .await
            .expect("done");
        create(&db, create_input(&project.id, "Open"))
            .await
            .expect("open");
        move_to(&db, &done.id, TaskStatus::Done, None, None)
            .await
            .expect("move");

        let now = crate::db::now_ms();
        assert_eq!(
            ids(&completed(&db, now - 60_000, now + 60_000)
                .await
                .expect("week")),
            ["Done"]
        );
        assert!(
            completed(&db, now + 60_000, now + 120_000)
                .await
                .expect("later")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn focus_blocks_overlap_the_range_and_skip_breaks() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        for (id, kind, started, ended) in [
            ("in", "focus", 1_000, 2_000),
            ("edge", "manual", 500, 1_500),
            ("break", "break", 1_000, 2_000),
            ("before", "focus", 0, 1_000),
        ] {
            sqlx::query(
                "INSERT INTO time_entries
                    (id, created_at, updated_at, device_id, project_id, kind, started_at,
                     ended_at, duration_s)
                 VALUES (?, 0, 1, 'd', ?, ?, ?, ?, 1)",
            )
            .bind(id)
            .bind(&project.id)
            .bind(kind)
            .bind(started)
            .bind(ended)
            .execute(&db.pool)
            .await
            .expect("entry");
        }

        let blocks = focus_blocks(&db, 1_000, 3_000).await.expect("blocks");

        let found: Vec<&str> = blocks.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(found, ["edge", "in"]);
        assert!(focus_blocks(&db, 3_000, 3_000).await.is_err());
    }

    #[tokio::test]
    async fn available_hours_default_and_change() {
        let db = Db::in_memory().await;
        assert_eq!(available_minutes(&db).await.expect("default"), 360);
        assert_eq!(set_available_minutes(&db, 300).await.expect("set"), 300);
        assert_eq!(available_minutes(&db).await.expect("stored"), 300);
        assert!(set_available_minutes(&db, 10).await.is_err());
    }
}
