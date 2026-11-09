use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use crate::domain::{Task, TaskCreate, TaskStatus, TaskUpdate, order_key, tags};
use crate::error::{AppError, AppResult};
use sqlx::SqliteConnection;
use sqlx::types::Json;

pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<Task>> {
    let rows = sqlx::query_as(
        "SELECT * FROM tasks WHERE project_id = ? AND deleted = 0 ORDER BY order_key",
    )
    .bind(project_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<Task> {
    sqlx::query_as("SELECT * FROM tasks WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("task"))
}

/// New tasks start in To do, at the end of their siblings.
pub async fn create(db: &Db, input: TaskCreate) -> AppResult<Task> {
    let mut tx = db.pool.begin().await?;
    let id = insert(&mut tx, db, &input).await?;
    tx.commit().await?;
    get(db, &id).await
}

// takes a connection so callers can add a task inside their own transaction
pub(super) async fn insert(
    conn: &mut SqliteConnection,
    db: &Db,
    input: &TaskCreate,
) -> AppResult<String> {
    let title = clean_title(&input.title)?;
    let id = new_id();
    let last = last_key(
        conn,
        &input.project_id,
        input.parent_id.as_deref(),
        TaskStatus::Todo,
        None,
    )
    .await?;
    sqlx::query(
        "INSERT INTO tasks
            (id, created_at, updated_at, device_id, project_id, parent_id, title, body_md,
             acceptance_md, related_files, priority, due_at, order_key, tags)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(&input.project_id)
    .bind(&input.parent_id)
    .bind(&title)
    .bind(&input.body_md)
    .bind(&input.acceptance_md)
    .bind(Json(tags::clean(input.related_files.clone())))
    .bind(input.priority)
    .bind(input.due_at)
    .bind(order_key::between(last.as_deref().unwrap_or(""), None))
    .bind(Json(tags::clean(input.tags.clone())))
    .execute(&mut *conn)
    .await?;
    activity::record(
        conn,
        db,
        "task.created",
        "task",
        &id,
        &format!("Added task {title}"),
    )
    .await?;
    Ok(id)
}

pub async fn update(db: &Db, id: &str, input: TaskUpdate) -> AppResult<Task> {
    let title = clean_title(&input.title)?;
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE tasks
            SET title = ?, body_md = ?, acceptance_md = ?, related_files = ?, priority = ?,
                due_at = ?, tags = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(&title)
    .bind(&input.body_md)
    .bind(&input.acceptance_md)
    .bind(Json(tags::clean(input.related_files)))
    .bind(input.priority)
    .bind(input.due_at)
    .bind(Json(tags::clean(input.tags)))
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "task")?;
    activity::record(
        &mut tx,
        db,
        "task.updated",
        "task",
        id,
        &format!("Updated task {title}"),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

/// Soft deletes the task with its subtasks and checklist items.
pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE tasks SET deleted = 1, updated_at = ?, device_id = ?
          WHERE (id = ? OR parent_id = ?) AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "task")?;
    sqlx::query(
        "UPDATE checklist_items SET deleted = 1, updated_at = ?, device_id = ?
          WHERE deleted = 0
            AND task_id IN (SELECT id FROM tasks WHERE id = ? OR parent_id = ?)",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    activity::record(&mut tx, db, "task.deleted", "task", id, "Deleted task").await?;
    tx.commit().await?;
    Ok(())
}

/// Puts the task in `status`, right before `before_id`, or last when it is `None`.
pub async fn move_to(
    db: &Db,
    id: &str,
    status: TaskStatus,
    before_id: Option<&str>,
) -> AppResult<Task> {
    let mut tx = db.pool.begin().await?;
    move_in(&mut tx, db, id, status, before_id).await?;
    tx.commit().await?;
    get(db, id).await
}

// takes a connection so callers can move a task inside their own transaction
pub(super) async fn move_in(
    conn: &mut SqliteConnection,
    db: &Db,
    id: &str,
    status: TaskStatus,
    before_id: Option<&str>,
) -> AppResult<()> {
    let (project_id, parent_id): (String, Option<String>) =
        sqlx::query_as("SELECT project_id, parent_id FROM tasks WHERE id = ? AND deleted = 0")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or(AppError::NotFound("task"))?;

    let next_key: Option<String> = match before_id {
        Some(before_id) => Some(
            sqlx::query_scalar("SELECT order_key FROM tasks WHERE id = ? AND deleted = 0")
                .bind(before_id)
                .fetch_optional(&mut *conn)
                .await?
                .ok_or(AppError::NotFound("task"))?,
        ),
        None => None,
    };
    let previous_key = last_key(
        conn,
        &project_id,
        parent_id.as_deref(),
        status,
        Some((id, next_key.as_deref())),
    )
    .await?;
    let key = order_key::between(previous_key.as_deref().unwrap_or(""), next_key.as_deref());

    sqlx::query(
        "UPDATE tasks SET status = ?, order_key = ?, updated_at = ?, device_id = ?
          WHERE id = ?",
    )
    .bind(status)
    .bind(key)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *conn)
    .await?;
    activity::record(
        conn,
        db,
        "task.moved",
        "task",
        id,
        &format!("Moved task to {}", status.as_str()),
    )
    .await?;
    Ok(())
}

fn clean_title(title: &str) -> AppResult<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("task title is required"));
    }
    Ok(title.to_string())
}

// greatest key among the siblings in a column; `skip` leaves out the task being
// moved and keeps only keys below the given upper bound
async fn last_key(
    conn: &mut SqliteConnection,
    project_id: &str,
    parent_id: Option<&str>,
    status: TaskStatus,
    skip: Option<(&str, Option<&str>)>,
) -> AppResult<Option<String>> {
    let (skip_id, below) = skip.unwrap_or(("", None));
    let key = sqlx::query_scalar(
        "SELECT MAX(order_key) FROM tasks
          WHERE project_id = ? AND parent_id IS ? AND status = ? AND deleted = 0
            AND id <> ? AND (? IS NULL OR order_key < ?)",
    )
    .bind(project_id)
    .bind(parent_id)
    .bind(status)
    .bind(skip_id)
    .bind(below)
    .bind(below)
    .fetch_one(conn)
    .await?;
    Ok(key)
}

#[cfg(test)]
pub fn create_input(project_id: &str, title: &str) -> TaskCreate {
    use crate::domain::task::Priority;
    TaskCreate {
        project_id: project_id.to_string(),
        parent_id: None,
        title: title.to_string(),
        body_md: String::new(),
        acceptance_md: String::new(),
        related_files: vec![],
        priority: Priority::Medium,
        due_at: None,
        tags: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::{self, create_for_test};
    use crate::domain::task::Priority;

    async fn titles(db: &Db, project_id: &str, status: TaskStatus) -> Vec<String> {
        list(db, project_id)
            .await
            .expect("list")
            .into_iter()
            .filter(|t| t.status == status && t.parent_id.is_none())
            .map(|t| t.title)
            .collect()
    }

    #[tokio::test]
    async fn create_appends_in_order_and_records_activity() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let a = create(&db, create_input(&project.id, "A"))
            .await
            .expect("a");
        create(&db, create_input(&project.id, "B"))
            .await
            .expect("b");

        assert_eq!(a.status, TaskStatus::Todo);
        assert_eq!(titles(&db, &project.id, TaskStatus::Todo).await, ["A", "B"]);
        assert_eq!(kinds_for(&db, &a.id).await, ["task.created"]);
    }

    #[tokio::test]
    async fn create_rejects_blank_title() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let result = create(&db, create_input(&project.id, " ")).await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn update_changes_fields() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let task = create(&db, create_input(&project.id, "A"))
            .await
            .expect("a");

        let updated = update(
            &db,
            &task.id,
            TaskUpdate {
                title: "A2".into(),
                body_md: "body".into(),
                acceptance_md: "- works".into(),
                related_files: vec!["src/main.rs".into()],
                priority: Priority::High,
                due_at: Some(1_700_000_000_000),
                tags: vec!["ui".into()],
            },
        )
        .await
        .expect("update");

        assert_eq!(updated.title, "A2");
        assert_eq!(updated.priority, Priority::High);
        assert_eq!(updated.due_at, Some(1_700_000_000_000));
        assert_eq!(updated.related_files.0, vec!["src/main.rs"]);
        assert_eq!(updated.tags.0, vec!["ui"]);
    }

    #[tokio::test]
    async fn move_places_task_before_another_in_a_new_column() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let a = create(&db, create_input(&project.id, "A"))
            .await
            .expect("a");
        let b = create(&db, create_input(&project.id, "B"))
            .await
            .expect("b");
        let c = create(&db, create_input(&project.id, "C"))
            .await
            .expect("c");

        move_to(&db, &a.id, TaskStatus::Doing, None)
            .await
            .expect("a");
        move_to(&db, &b.id, TaskStatus::Doing, None)
            .await
            .expect("b");
        move_to(&db, &c.id, TaskStatus::Doing, Some(&b.id))
            .await
            .expect("c");

        assert_eq!(
            titles(&db, &project.id, TaskStatus::Doing).await,
            ["A", "C", "B"]
        );
        assert!(titles(&db, &project.id, TaskStatus::Todo).await.is_empty());
        assert_eq!(kinds_for(&db, &c.id).await, ["task.created", "task.moved"]);
    }

    #[tokio::test]
    async fn move_within_a_column_reorders_without_touching_others() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let a = create(&db, create_input(&project.id, "A"))
            .await
            .expect("a");
        let b = create(&db, create_input(&project.id, "B"))
            .await
            .expect("b");
        let c = create(&db, create_input(&project.id, "C"))
            .await
            .expect("c");

        move_to(&db, &c.id, TaskStatus::Todo, Some(&a.id))
            .await
            .expect("to front");
        assert_eq!(
            titles(&db, &project.id, TaskStatus::Todo).await,
            ["C", "A", "B"]
        );

        move_to(&db, &c.id, TaskStatus::Todo, None)
            .await
            .expect("to end");
        assert_eq!(
            titles(&db, &project.id, TaskStatus::Todo).await,
            ["A", "B", "C"]
        );

        let untouched = get(&db, &b.id).await.expect("b");
        assert_eq!(untouched.order_key, b.order_key);
    }

    #[tokio::test]
    async fn subtasks_are_ordered_separately_from_their_parent_column() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let parent = create(&db, create_input(&project.id, "Parent"))
            .await
            .expect("parent");
        let mut sub = create_input(&project.id, "Sub");
        sub.parent_id = Some(parent.id.clone());
        let sub = create(&db, sub).await.expect("sub");

        assert_eq!(sub.parent_id.as_deref(), Some(parent.id.as_str()));
        assert_eq!(titles(&db, &project.id, TaskStatus::Todo).await, ["Parent"]);
    }

    #[tokio::test]
    async fn delete_cascades_to_subtasks_and_checklist() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let parent = create(&db, create_input(&project.id, "Parent"))
            .await
            .expect("parent");
        let mut sub = create_input(&project.id, "Sub");
        sub.parent_id = Some(parent.id.clone());
        create(&db, sub).await.expect("sub");
        crate::db::checklist::create(&db, &parent.id, "step")
            .await
            .expect("item");

        delete(&db, &parent.id).await.expect("delete");

        assert!(list(&db, &project.id).await.expect("list").is_empty());
        let live_items: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM checklist_items WHERE deleted = 0")
                .fetch_one(&db.pool)
                .await
                .expect("count");
        assert_eq!(live_items, 0);
        assert!(matches!(
            get(&db, &parent.id).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn progress_counts_only_top_level_done_tasks() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let a = create(&db, create_input(&project.id, "A"))
            .await
            .expect("a");
        create(&db, create_input(&project.id, "B"))
            .await
            .expect("b");
        move_to(&db, &a.id, TaskStatus::Done, None)
            .await
            .expect("done");

        let summary = projects::get(&db, &project.id).await.expect("get");
        assert_eq!((summary.tasks_done, summary.tasks_total), (1, 2));
    }
}
