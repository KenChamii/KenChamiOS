mod activity;
pub mod assets;
pub mod checklist;
pub mod docs;
mod handle;
pub mod media;
pub mod metadata_cache;
pub mod projects;
pub mod prompts;
pub mod save_points;
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
