use crate::error::{AppError, AppResult};
use crate::sync::hlc::Hlc;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteQueryResult;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Everything a repository needs to read and write rows.
pub struct Db {
    pub pool: SqlitePool,
    pub hlc: Hlc,
    pub device_id: String,
}

impl Db {
    pub fn new(pool: SqlitePool, hlc: Hlc, device_id: String) -> Self {
        Self {
            pool,
            hlc,
            device_id,
        }
    }

    /// Next `updated_at` value for a write.
    pub fn stamp(&self) -> i64 {
        // an hlc value is around 1e17, well inside i64
        self.hlc.now() as i64
    }

    #[cfg(test)]
    pub async fn in_memory() -> Self {
        let pool = super::open_in_memory().await.expect("open db");
        Self::new(pool, Hlc::new(), "test-device".to_string())
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

pub fn require_changed(result: SqliteQueryResult, what: &'static str) -> AppResult<()> {
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(what));
    }
    Ok(())
}
