use crate::db::snapshot;
use crate::error::AppResult;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};

const PREFIX: &str = "pre-sync-";
const KEEP: usize = 5;

/// A copy of the database taken before a sync batch is applied; the newest few are kept.
pub async fn create(pool: &SqlitePool, dir: &Path) -> AppResult<PathBuf> {
    snapshot::create(pool, dir, PREFIX, KEEP).await
}
