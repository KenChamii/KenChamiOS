use crate::error::AppResult;
use crate::sync::hlc::Hlc;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const DEVICE_ID_KEY: &str = "device_id";

/// Returns the device id, creating and storing it on the first call.
pub async fn ensure_device_id(pool: &SqlitePool, hlc: &Hlc) -> AppResult<String> {
    let candidate = Uuid::now_v7().to_string();
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64);

    sqlx::query(
        "INSERT INTO settings (id, created_at, updated_at, device_id, key, value_json)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(key) DO NOTHING",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(now_ms)
    // an hlc value is around 1e17, well inside i64
    .bind(hlc.now() as i64)
    .bind(&candidate)
    .bind(DEVICE_ID_KEY)
    .bind(serde_json::to_string(&candidate)?)
    .execute(pool)
    .await?;

    let stored: String = sqlx::query_scalar("SELECT value_json FROM settings WHERE key = ?")
        .bind(DEVICE_ID_KEY)
        .fetch_one(pool)
        .await?;
    Ok(serde_json::from_str(&stored)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn device_id_is_created_once() {
        let pool = open_in_memory().await.expect("open db");
        let hlc = Hlc::new();

        let first = ensure_device_id(&pool, &hlc).await.expect("first call");
        let second = ensure_device_id(&pool, &hlc).await.expect("second call");

        assert_eq!(first, second);
        assert!(Uuid::parse_str(&first).is_ok());
    }
}
