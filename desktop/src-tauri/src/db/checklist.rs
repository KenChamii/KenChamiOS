use super::activity;
use super::handle::{Db, new_id, now_ms};
use crate::domain::{ChecklistItem, order_key};
use crate::error::{AppError, AppResult};

pub async fn list(db: &Db, task_id: &str) -> AppResult<Vec<ChecklistItem>> {
    let rows = sqlx::query_as(
        "SELECT * FROM checklist_items WHERE task_id = ? AND deleted = 0 ORDER BY order_key",
    )
    .bind(task_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn create(db: &Db, task_id: &str, text: &str) -> AppResult<ChecklistItem> {
    let text = clean_text(text)?;
    let id = new_id();
    let mut tx = db.pool.begin().await?;
    let last: Option<String> = sqlx::query_scalar(
        "SELECT MAX(order_key) FROM checklist_items WHERE task_id = ? AND deleted = 0",
    )
    .bind(task_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO checklist_items
            (id, created_at, updated_at, device_id, task_id, text, order_key)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(task_id)
    .bind(&text)
    .bind(order_key::between(last.as_deref().unwrap_or(""), None))
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "checklist.created",
        "task",
        task_id,
        &format!("Added checklist item {text}"),
    )
    .await?;
    tx.commit().await?;
    get(db, &id).await
}

pub async fn update(db: &Db, id: &str, text: &str, done: bool) -> AppResult<ChecklistItem> {
    let text = clean_text(text)?;
    let mut tx = db.pool.begin().await?;
    let task_id: String = sqlx::query_scalar(
        "UPDATE checklist_items SET text = ?, done = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0
      RETURNING task_id",
    )
    .bind(&text)
    .bind(done)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound("checklist item"))?;
    let kind = if done {
        "checklist.checked"
    } else {
        "checklist.updated"
    };
    activity::record(
        &mut tx,
        db,
        kind,
        "task",
        &task_id,
        &format!("Checklist: {text}"),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let task_id: String = sqlx::query_scalar(
        "UPDATE checklist_items SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0
      RETURNING task_id",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound("checklist item"))?;
    activity::record(
        &mut tx,
        db,
        "checklist.deleted",
        "task",
        &task_id,
        "Removed checklist item",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn get(db: &Db, id: &str) -> AppResult<ChecklistItem> {
    sqlx::query_as("SELECT * FROM checklist_items WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("checklist item"))
}

fn clean_text(text: &str) -> AppResult<String> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Invalid("checklist text is required"));
    }
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{self, create_input};

    async fn task_id(db: &Db) -> String {
        let project = create_for_test(db, "P").await;
        tasks::create(db, create_input(&project.id, "T"))
            .await
            .expect("task")
            .id
    }

    #[tokio::test]
    async fn items_keep_creation_order() {
        let db = Db::in_memory().await;
        let task = task_id(&db).await;
        create(&db, &task, "one").await.expect("one");
        create(&db, &task, "two").await.expect("two");
        create(&db, &task, "three").await.expect("three");

        let texts: Vec<String> = list(&db, &task)
            .await
            .expect("list")
            .into_iter()
            .map(|i| i.text)
            .collect();
        assert_eq!(texts, ["one", "two", "three"]);
    }

    #[tokio::test]
    async fn update_checks_item_and_records_activity_on_the_task() {
        let db = Db::in_memory().await;
        let task = task_id(&db).await;
        let item = create(&db, &task, "one").await.expect("one");
        assert!(!item.done);

        let updated = update(&db, &item.id, "one!", true).await.expect("update");

        assert!(updated.done);
        assert_eq!(updated.text, "one!");
        assert_eq!(
            kinds_for(&db, &task).await,
            ["task.created", "checklist.created", "checklist.checked"]
        );
    }

    #[tokio::test]
    async fn delete_is_soft() {
        let db = Db::in_memory().await;
        let task = task_id(&db).await;
        let item = create(&db, &task, "one").await.expect("one");

        delete(&db, &item.id).await.expect("delete");

        assert!(list(&db, &task).await.expect("list").is_empty());
        let stored: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM checklist_items WHERE deleted = 1")
                .fetch_one(&db.pool)
                .await
                .expect("count");
        assert_eq!(stored, 1);
        assert!(matches!(
            delete(&db, &item.id).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn blank_text_is_rejected() {
        let db = Db::in_memory().await;
        let task = task_id(&db).await;
        assert!(matches!(
            create(&db, &task, "  ").await,
            Err(AppError::Invalid(_))
        ));
    }
}
