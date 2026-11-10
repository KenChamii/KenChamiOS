use super::handle::Db;
use super::settings;
use crate::error::AppResult;
use sqlx::FromRow;

const NOTIFIED_PREFIX: &str = "reminders.due_notified.";
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct DueReminder {
    pub task_id: String,
    pub title: String,
    pub project_name: String,
    pub due_at: i64,
}

/// Open tasks that fell due within the last day and were not announced for that due date yet.
/// They are marked as announced, so a task is announced again only when its due date changes.
pub async fn take(db: &Db, now: i64) -> AppResult<Vec<DueReminder>> {
    // settings hold json text, so the stored due date is a quoted number
    let due: Vec<DueReminder> = sqlx::query_as(
        "SELECT t.id AS task_id, t.title, p.name AS project_name, t.due_at
           FROM tasks t JOIN projects p ON p.id = t.project_id
          WHERE t.deleted = 0 AND t.status != 'done' AND t.due_at IS NOT NULL
            AND t.due_at <= ?1 AND t.due_at > ?1 - ?2
            AND p.deleted = 0 AND p.archived = 0
            AND NOT EXISTS (SELECT 1 FROM settings s
                             WHERE s.key = ?3 || t.id AND s.deleted = 0
                               AND s.value_json = json_quote(CAST(t.due_at AS TEXT)))
          ORDER BY t.due_at, t.id",
    )
    .bind(now)
    .bind(DAY_MS)
    .bind(NOTIFIED_PREFIX)
    .fetch_all(&db.pool)
    .await?;
    for reminder in &due {
        let key = format!("{NOTIFIED_PREFIX}{}", reminder.task_id);
        settings::set_string(db, &key, &reminder.due_at.to_string()).await?;
    }
    Ok(due)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;
    use crate::db::tasks;
    use crate::domain::task::TaskStatus;

    async fn due_task(db: &Db, project_id: &str, title: &str, due_at: i64) -> String {
        let mut input = tasks::create_input(project_id, title);
        input.due_at = Some(due_at);
        tasks::create(db, input).await.expect("task").id
    }

    fn titles(reminders: &[DueReminder]) -> Vec<&str> {
        reminders.iter().map(|r| r.title.as_str()).collect()
    }

    #[tokio::test]
    async fn announces_each_due_date_once() {
        let db = Db::in_memory().await;
        let now = 100 * DAY_MS;
        let project = create_for_test(&db, "P").await;
        let today = due_task(&db, &project.id, "Today", now - 1_000).await;
        due_task(&db, &project.id, "Long overdue", now - 3 * DAY_MS).await;
        due_task(&db, &project.id, "Tomorrow", now + DAY_MS).await;
        let done = due_task(&db, &project.id, "Done", now - 1_000).await;
        tasks::move_to(&db, &done, TaskStatus::Done, None, None)
            .await
            .expect("done");

        let first = take(&db, now).await.expect("first");
        assert_eq!(titles(&first), ["Today"]);
        assert_eq!(first[0].project_name, "P");
        assert!(take(&db, now + 60_000).await.expect("again").is_empty());

        // a new due date is a new reminder
        let mut input = crate::domain::TaskUpdate {
            title: "Today".into(),
            body_md: String::new(),
            acceptance_md: String::new(),
            related_files: vec![],
            priority: crate::domain::task::Priority::Medium,
            due_at: Some(now + DAY_MS - 1_000),
            tags: vec![],
            recurrence: None,
        };
        tasks::update(&db, &today, input.clone())
            .await
            .expect("move");
        assert!(take(&db, now).await.expect("not yet").is_empty());
        input.due_at = Some(now + DAY_MS);
        tasks::update(&db, &today, input).await.expect("move");
        let later = take(&db, now + DAY_MS).await.expect("later");
        assert_eq!(titles(&later), ["Today", "Tomorrow"]);
    }
}
