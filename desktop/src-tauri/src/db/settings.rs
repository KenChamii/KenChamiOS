use super::handle::{Db, new_id, now_ms};
use crate::error::AppResult;
use crate::sync::hlc::Hlc;
use sqlx::SqlitePool;
use uuid::Uuid;

const DEVICE_ID_KEY: &str = "device_id";

/// Returns the device id, creating and storing it on the first call.
pub async fn ensure_device_id(pool: &SqlitePool, hlc: &Hlc) -> AppResult<String> {
    let candidate = Uuid::now_v7().to_string();

    sqlx::query(
        "INSERT INTO settings (id, created_at, updated_at, device_id, key, value_json)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(key) DO NOTHING",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(now_ms())
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

/// A stored string setting; `None` while it was never set.
pub async fn get_string(db: &Db, key: &str) -> AppResult<Option<String>> {
    let stored: Option<String> =
        sqlx::query_scalar("SELECT value_json FROM settings WHERE key = ? AND deleted = 0")
            .bind(key)
            .fetch_optional(&db.pool)
            .await?;
    Ok(stored.map(|json| serde_json::from_str(&json)).transpose()?)
}

pub async fn set_string(db: &Db, key: &str, value: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO settings (id, created_at, updated_at, device_id, key, value_json)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(key) DO UPDATE
            SET value_json = excluded.value_json, updated_at = excluded.updated_at,
                device_id = excluded.device_id, deleted = 0",
    )
    .bind(new_id())
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(key)
    .bind(serde_json::to_string(value)?)
    .execute(&db.pool)
    .await?;
    Ok(())
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

    #[tokio::test]
    async fn string_settings_are_stored_and_replaced() {
        let db = Db::in_memory().await;
        assert_eq!(get_string(&db, "k").await.expect("get"), None);

        set_string(&db, "k", "one").await.expect("set");
        set_string(&db, "k", "two").await.expect("replace");

        assert_eq!(
            get_string(&db, "k").await.expect("get").as_deref(),
            Some("two")
        );
    }
}
