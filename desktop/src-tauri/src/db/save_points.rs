use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use super::projects;
use crate::domain::{SavePoint, SavePointInput};
use crate::error::{AppError, AppResult};

/// Newest first.
pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<SavePoint>> {
    let rows = sqlx::query_as(
        "SELECT * FROM save_points WHERE project_id = ? AND deleted = 0
          ORDER BY created_at DESC, id DESC",
    )
    .bind(project_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// Stores a manual save point at the project's current stage.
pub async fn create(
    db: &Db,
    project_id: &str,
    input: SavePointInput,
    git_branch: Option<&str>,
    git_commit: Option<&str>,
) -> AppResult<SavePoint> {
    let note = input.note.trim();
    if note.is_empty() {
        return Err(AppError::Invalid("save point note is required"));
    }
    let stage = projects::get(db, project_id).await?.project.stage;
    let id = new_id();
    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO save_points
            (id, created_at, updated_at, device_id, project_id, stage, note,
             done_md, next_md, git_branch, git_commit)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(project_id)
    .bind(stage)
    .bind(note)
    .bind(input.done_md.trim())
    .bind(input.next_md.trim())
    .bind(git_branch)
    .bind(git_commit)
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "save_point.created",
        "save_point",
        &id,
        &format!("Save point: {note}"),
    )
    .await?;
    let created = sqlx::query_as("SELECT * FROM save_points WHERE id = ?")
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(created)
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE save_points SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "save point")?;
    activity::record(
        &mut tx,
        db,
        "save_point.deleted",
        "save_point",
        id,
        "Deleted save point",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::{create_for_test, set_stage};
    use crate::domain::Stage;

    fn input(note: &str) -> SavePointInput {
        SavePointInput {
            note: note.to_string(),
            done_md: " - done ".to_string(),
            next_md: "- next".to_string(),
        }
    }

    async fn add(db: &Db, project_id: &str, note: &str) -> SavePoint {
        create(db, project_id, input(note), None, None)
            .await
            .expect("create save point")
    }

    #[tokio::test]
    async fn create_uses_project_stage_and_given_commit() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        set_stage(&db, &project.id, Stage::Build)
            .await
            .expect("stage");

        let point = create(
            &db,
            &project.id,
            input(" Stopped here "),
            Some("main"),
            Some("abc123"),
        )
        .await
        .expect("create");

        assert_eq!(point.note, "Stopped here");
        assert_eq!(point.stage, Stage::Build);
        assert_eq!(point.source, "manual");
        assert_eq!(point.done_md, "- done");
        assert_eq!(point.git_branch.as_deref(), Some("main"));
        assert_eq!(point.git_commit.as_deref(), Some("abc123"));
        assert_eq!(kinds_for(&db, &point.id).await, ["save_point.created"]);
    }

    #[tokio::test]
    async fn create_without_git_leaves_columns_empty() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let point = add(&db, &project.id, "n").await;
        assert_eq!(point.git_branch, None);
        assert_eq!(point.git_commit, None);
    }

    #[tokio::test]
    async fn create_rejects_blank_note_and_unknown_project() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let blank = create(&db, &project.id, input("  "), None, None).await;
        assert!(matches!(blank, Err(AppError::Invalid(_))));
        let missing = create(&db, "missing", input("n"), None, None).await;
        assert!(matches!(missing, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn list_is_newest_first_and_per_project() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let other = create_for_test(&db, "Q").await;
        add(&db, &project.id, "first").await;
        add(&db, &project.id, "second").await;
        add(&db, &other.id, "other").await;

        let notes: Vec<String> = list(&db, &project.id)
            .await
            .expect("list")
            .into_iter()
            .map(|p| p.note)
            .collect();
        assert_eq!(notes, ["second", "first"]);
    }

    #[tokio::test]
    async fn delete_hides_the_save_point() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let point = add(&db, &project.id, "n").await;

        delete(&db, &point.id).await.expect("delete");

        assert!(list(&db, &project.id).await.expect("list").is_empty());
        assert!(matches!(
            delete(&db, &point.id).await,
            Err(AppError::NotFound(_))
        ));
        assert_eq!(
            kinds_for(&db, &point.id).await,
            ["save_point.created", "save_point.deleted"]
        );
    }

    #[tokio::test]
    async fn project_delete_removes_its_save_points() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        add(&db, &project.id, "n").await;

        projects::delete(&db, &project.id).await.expect("delete");

        assert!(list(&db, &project.id).await.expect("list").is_empty());
    }
}
