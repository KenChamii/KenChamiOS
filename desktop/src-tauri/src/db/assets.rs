use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use super::projects;
use crate::domain::asset::{AssetKind, ProjectAsset};
use crate::domain::order_key;
use crate::error::{AppError, AppResult};

pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<ProjectAsset>> {
    let rows = sqlx::query_as(
        "SELECT * FROM project_assets WHERE project_id = ? AND deleted = 0 ORDER BY order_key",
    )
    .bind(project_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<ProjectAsset> {
    sqlx::query_as("SELECT * FROM project_assets WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("asset"))
}

/// Adds a screenshot or link at the end of the project's assets.
pub async fn add(
    db: &Db,
    project_id: &str,
    kind: AssetKind,
    path_or_url: &str,
    caption: &str,
) -> AppResult<ProjectAsset> {
    projects::get(db, project_id).await?;
    let id = new_id();
    let mut tx = db.pool.begin().await?;
    let last: Option<String> = sqlx::query_scalar(
        "SELECT MAX(order_key) FROM project_assets WHERE project_id = ? AND deleted = 0",
    )
    .bind(project_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO project_assets
            (id, created_at, updated_at, device_id, project_id, kind, path_or_url,
             caption, order_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(project_id)
    .bind(kind)
    .bind(path_or_url)
    .bind(caption.trim())
    .bind(order_key::between(last.as_deref().unwrap_or(""), None))
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "asset.added",
        "project_asset",
        &id,
        match kind {
            AssetKind::Screenshot => "Added a screenshot",
            AssetKind::Link => "Added a link",
        },
    )
    .await?;
    tx.commit().await?;
    get(db, &id).await
}

pub async fn set_caption(db: &Db, id: &str, caption: &str) -> AppResult<ProjectAsset> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE project_assets SET caption = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(caption.trim())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "asset")?;
    activity::record(
        &mut tx,
        db,
        "asset.updated",
        "project_asset",
        id,
        "Edited a caption",
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE project_assets SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "asset")?;
    activity::record(
        &mut tx,
        db,
        "asset.deleted",
        "project_asset",
        id,
        "Removed an asset",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;

    #[tokio::test]
    async fn assets_keep_the_order_they_were_added_in() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        for url in ["https://a.test", "https://b.test", "https://c.test"] {
            add(&db, &project.id, AssetKind::Link, url, " Site ")
                .await
                .expect("add");
        }

        let assets = list(&db, &project.id).await.expect("list");

        let urls: Vec<&str> = assets.iter().map(|a| a.path_or_url.as_str()).collect();
        assert_eq!(urls, ["https://a.test", "https://b.test", "https://c.test"]);
        assert_eq!(assets[0].caption, "Site");
    }

    #[tokio::test]
    async fn caption_edit_and_delete_are_logged() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        let asset = add(&db, &project.id, AssetKind::Screenshot, "a.png", "")
            .await
            .expect("add");

        let edited = set_caption(&db, &asset.id, "Home screen")
            .await
            .expect("caption");
        assert_eq!(edited.caption, "Home screen");

        delete(&db, &asset.id).await.expect("delete");
        assert!(list(&db, &project.id).await.expect("list").is_empty());
        assert!(matches!(
            delete(&db, &asset.id).await,
            Err(AppError::NotFound(_))
        ));
        assert_eq!(
            kinds_for(&db, &asset.id).await,
            ["asset.added", "asset.updated", "asset.deleted"]
        );
    }

    #[tokio::test]
    async fn unknown_project_is_rejected_and_project_delete_hides_assets() {
        let db = Db::in_memory().await;
        let missing = add(&db, "nope", AssetKind::Link, "https://a.test", "").await;
        assert!(matches!(missing, Err(AppError::NotFound(_))));

        let project = create_for_test(&db, "Demo").await;
        add(&db, &project.id, AssetKind::Link, "https://a.test", "")
            .await
            .expect("add");
        projects::delete(&db, &project.id).await.expect("delete");
        assert!(list(&db, &project.id).await.expect("list").is_empty());
    }
}
