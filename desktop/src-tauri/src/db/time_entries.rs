use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use super::tasks;
use crate::domain::time_entry::{EntryKind, TimeEntry, TimeEntryInput};
use crate::error::{AppError, AppResult};
use sqlx::SqliteConnection;

const MAX_DURATION_S: i64 = 24 * 60 * 60;

/// Newest first.
pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<TimeEntry>> {
    let rows = sqlx::query_as(
        "SELECT * FROM time_entries WHERE project_id = ? AND deleted = 0
          ORDER BY started_at DESC, id DESC",
    )
    .bind(project_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// A hand-written entry on a project, optionally tied to one of its tasks.
pub async fn create_manual(
    db: &Db,
    project_id: &str,
    input: TimeEntryInput,
) -> AppResult<TimeEntry> {
    check(db, project_id, &input).await?;
    let mut tx = db.pool.begin().await?;
    let id = insert(
        &mut tx,
        db,
        &NewEntry {
            project_id,
            task_id: input.task_id.as_deref(),
            kind: EntryKind::Manual,
            started_at: input.started_at,
            ended_at: input.started_at + input.duration_s * 1000,
            duration_s: input.duration_s,
            note: input.note.trim(),
        },
    )
    .await?;
    tx.commit().await?;
    get(db, &id).await
}

pub async fn update(db: &Db, id: &str, input: TimeEntryInput) -> AppResult<TimeEntry> {
    let existing = get(db, id).await?;
    check(db, &existing.project_id, &input).await?;
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE time_entries
            SET task_id = ?, started_at = ?, ended_at = ?, duration_s = ?, note = ?,
                updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(&input.task_id)
    .bind(input.started_at)
    .bind(input.started_at + input.duration_s * 1000)
    .bind(input.duration_s)
    .bind(input.note.trim())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "time entry")?;
    activity::record(
        &mut tx,
        db,
        "time.updated",
        "time_entry",
        id,
        "Edited a time entry",
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE time_entries SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "time entry")?;
    activity::record(
        &mut tx,
        db,
        "time.deleted",
        "time_entry",
        id,
        "Deleted a time entry",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) struct NewEntry<'a> {
    pub project_id: &'a str,
    pub task_id: Option<&'a str>,
    pub kind: EntryKind,
    pub started_at: i64,
    pub ended_at: i64,
    pub duration_s: i64,
    pub note: &'a str,
}

// takes a connection so the timer can log a finished phase with its own state change
pub(crate) async fn insert(
    conn: &mut SqliteConnection,
    db: &Db,
    entry: &NewEntry<'_>,
) -> AppResult<String> {
    let id = new_id();
    sqlx::query(
        "INSERT INTO time_entries
            (id, created_at, updated_at, device_id, project_id, task_id, kind,
             started_at, ended_at, duration_s, note)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(entry.project_id)
    .bind(entry.task_id)
    .bind(entry.kind)
    .bind(entry.started_at)
    .bind(entry.ended_at)
    .bind(entry.duration_s)
    .bind(entry.note)
    .execute(&mut *conn)
    .await?;
    activity::record(
        conn,
        db,
        "time.logged",
        "time_entry",
        &id,
        &format!("Logged {} min", entry.duration_s / 60),
    )
    .await?;
    Ok(id)
}

async fn get(db: &Db, id: &str) -> AppResult<TimeEntry> {
    sqlx::query_as("SELECT * FROM time_entries WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("time entry"))
}

async fn check(db: &Db, project_id: &str, input: &TimeEntryInput) -> AppResult<()> {
    if !(1..=MAX_DURATION_S).contains(&input.duration_s) {
        return Err(AppError::Invalid(
            "a time entry must be between 1 second and 24 hours",
        ));
    }
    if let Some(task_id) = &input.task_id
        && tasks::get(db, task_id).await?.project_id != project_id
    {
        return Err(AppError::Invalid("the task belongs to another project"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{create, create_input};

    fn input(task_id: Option<&str>, started_at: i64, duration_s: i64) -> TimeEntryInput {
        TimeEntryInput {
            task_id: task_id.map(str::to_string),
            started_at,
            duration_s,
            note: " pairing ".to_string(),
        }
    }

    #[tokio::test]
    async fn manual_entries_are_listed_newest_first() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        let task = create(&db, create_input(&project.id, "Do it"))
            .await
            .expect("task");
        create_manual(&db, &project.id, input(None, 1_000, 600))
            .await
            .expect("older");
        let newer = create_manual(&db, &project.id, input(Some(&task.id), 5_000, 1_800))
            .await
            .expect("newer");

        let entries = list(&db, &project.id).await.expect("list");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, newer.id);
        assert_eq!(entries[0].kind, EntryKind::Manual);
        assert_eq!(entries[0].ended_at, 5_000 + 1_800_000);
        assert_eq!(entries[0].note, "pairing");
        assert_eq!(entries[0].task_id.as_deref(), Some(task.id.as_str()));
        assert_eq!(kinds_for(&db, &newer.id).await, ["time.logged"]);
    }

    #[tokio::test]
    async fn edit_and_delete_change_the_list() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        let entry = create_manual(&db, &project.id, input(None, 0, 600))
            .await
            .expect("create");

        let edited = update(&db, &entry.id, input(None, 2_000, 900))
            .await
            .expect("update");
        assert_eq!((edited.started_at, edited.duration_s), (2_000, 900));
        assert_eq!(edited.ended_at, 2_000 + 900_000);

        delete(&db, &entry.id).await.expect("delete");
        assert!(list(&db, &project.id).await.expect("list").is_empty());
        assert!(matches!(
            delete(&db, &entry.id).await,
            Err(AppError::NotFound(_))
        ));
        assert_eq!(
            kinds_for(&db, &entry.id).await,
            ["time.logged", "time.updated", "time.deleted"]
        );
    }

    #[tokio::test]
    async fn deleting_the_project_hides_its_entries() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        create_manual(&db, &project.id, input(None, 0, 600))
            .await
            .expect("create");

        crate::db::projects::delete(&db, &project.id)
            .await
            .expect("delete project");

        assert!(list(&db, &project.id).await.expect("list").is_empty());
    }

    #[tokio::test]
    async fn invalid_entries_are_rejected() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        let other = create_for_test(&db, "Other").await;
        let foreign_task = create(&db, create_input(&other.id, "Elsewhere"))
            .await
            .expect("task");

        for bad in [
            input(None, 0, 0),
            input(None, 0, MAX_DURATION_S + 1),
            input(Some(&foreign_task.id), 0, 60),
        ] {
            let result = create_manual(&db, &project.id, bad).await;
            assert!(matches!(result, Err(AppError::Invalid(_))));
        }
        assert!(list(&db, &project.id).await.expect("list").is_empty());
    }
}
