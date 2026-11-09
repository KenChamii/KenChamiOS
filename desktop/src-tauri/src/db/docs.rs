use super::activity;
use super::handle::{Db, new_id, now_ms};
use crate::domain::{DocKind, ProjectDoc};
use crate::error::AppResult;

pub async fn get(db: &Db, project_id: &str, kind: DocKind) -> AppResult<Option<ProjectDoc>> {
    let doc = sqlx::query_as(
        "SELECT * FROM project_docs WHERE project_id = ? AND kind = ? AND deleted = 0",
    )
    .bind(project_id)
    .bind(kind)
    .fetch_optional(&db.pool)
    .await?;
    Ok(doc)
}

/// Creates the doc of this kind or replaces its title and body.
pub async fn upsert(
    db: &Db,
    project_id: &str,
    kind: DocKind,
    title: &str,
    body_md: &str,
) -> AppResult<ProjectDoc> {
    let mut tx = db.pool.begin().await?;
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT id FROM project_docs WHERE project_id = ? AND kind = ? AND deleted = 0",
    )
    .bind(project_id)
    .bind(kind)
    .fetch_optional(&mut *tx)
    .await?;

    let (id, verb) = match existing {
        Some(id) => {
            sqlx::query(
                "UPDATE project_docs SET title = ?, body_md = ?, updated_at = ?, device_id = ?
                  WHERE id = ?",
            )
            .bind(title)
            .bind(body_md)
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(&id)
            .execute(&mut *tx)
            .await?;
            (id, "Updated")
        }
        None => {
            let id = new_id();
            sqlx::query(
                "INSERT INTO project_docs
                    (id, created_at, updated_at, device_id, project_id, kind, title, body_md)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(now_ms())
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(project_id)
            .bind(kind)
            .bind(title)
            .bind(body_md)
            .execute(&mut *tx)
            .await?;
            (id, "Created")
        }
    };
    activity::record(
        &mut tx,
        db,
        "doc.saved",
        "doc",
        &id,
        &format!("{verb} {} doc", kind.as_str()),
    )
    .await?;
    let doc = sqlx::query_as("SELECT * FROM project_docs WHERE id = ?")
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;

    #[tokio::test]
    async fn get_is_empty_until_saved() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let doc = get(&db, &project.id, DocKind::Idea).await.expect("get");
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn upsert_creates_then_updates_the_same_doc() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;

        let first = upsert(&db, &project.id, DocKind::Plan, "Plan", "v1")
            .await
            .expect("create");
        let second = upsert(&db, &project.id, DocKind::Plan, "Plan", "v2")
            .await
            .expect("update");

        assert_eq!(first.id, second.id);
        assert_eq!(second.body_md, "v2");
        let stored = get(&db, &project.id, DocKind::Plan).await.expect("get");
        assert_eq!(stored.map(|d| d.body_md), Some("v2".to_string()));
        assert_eq!(kinds_for(&db, &first.id).await, ["doc.saved", "doc.saved"]);
    }

    #[tokio::test]
    async fn kinds_are_separate_docs() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        upsert(&db, &project.id, DocKind::Idea, "", "idea")
            .await
            .expect("idea");
        upsert(&db, &project.id, DocKind::Plan, "", "plan")
            .await
            .expect("plan");

        let idea = get(&db, &project.id, DocKind::Idea).await.expect("get");
        assert_eq!(idea.map(|d| d.body_md), Some("idea".to_string()));
    }
}
