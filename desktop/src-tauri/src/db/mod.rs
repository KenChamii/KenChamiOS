mod activity;
pub mod assets;
pub mod checklist;
pub mod dashboard;
pub mod docs;
mod handle;
pub mod media;
pub mod media_import;
pub mod metadata_cache;
pub mod paired_devices;
pub mod projects;
pub mod prompts;
pub mod save_points;
#[cfg(test)]
mod schema_contract;
pub mod session_notes;
pub mod settings;
pub mod tasks;
pub mod templates;
pub mod time_entries;
pub mod timer;

pub use handle::{Db, new_id, now_ms};

use crate::error::AppResult;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;

pub async fn open(path: &Path) -> AppResult<SqlitePool> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);
    connect(options, 5).await
}

async fn connect(options: SqliteConnectOptions, max_connections: u32) -> AppResult<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
pub async fn open_in_memory() -> AppResult<SqlitePool> {
    // one connection, because every in-memory connection is its own database
    connect(SqliteConnectOptions::new().in_memory(true), 1).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rebuilding_media_items_keeps_existing_rows_and_links() {
        let options = SqliteConnectOptions::new()
            .in_memory(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect");
        let mut old = sqlx::migrate!("./migrations");
        old.migrations = old
            .migrations
            .iter()
            .filter(|m| m.version < 9)
            .cloned()
            .collect();
        old.run(&pool).await.expect("old schema");
        for sql in [
            "INSERT INTO media_items (id, created_at, updated_at, device_id, type, title, source)
             VALUES ('i1', 1, 1, 'd', 'anime', 'Old', 'anilist')",
            "INSERT INTO media_entries (id, created_at, updated_at, device_id, media_item_id, status)
             VALUES ('e1', 1, 1, 'd', 'i1', 'planned')",
        ] {
            sqlx::query(sql).execute(&pool).await.expect("seed");
        }

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("upgrade");

        let title: String = sqlx::query_scalar(
            "SELECT i.title FROM media_entries e JOIN media_items i ON i.id = e.media_item_id",
        )
        .fetch_one(&pool)
        .await
        .expect("linked row");
        assert_eq!(title, "Old");
        sqlx::query(
            "INSERT INTO media_items (id, created_at, updated_at, device_id, type, title, source)
             VALUES ('i2', 1, 1, 'd', 'album', 'New', 'musicbrainz')",
        )
        .execute(&pool)
        .await
        .expect("new types are allowed");
    }
}
