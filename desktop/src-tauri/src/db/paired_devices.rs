use super::handle::{Db, now_ms};
use crate::error::AppResult;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PairedDevice {
    pub device_id: String,
    pub name: String,
    #[serde(skip)]
    pub token_hash: String,
    pub paired_at: i64,
    pub last_sync_at: Option<i64>,
    pub pulled_seq: i64,
}

/// Stores a device that just paired; pairing the same device again replaces its credential.
pub async fn add(db: &Db, device_id: &str, name: &str, token_hash: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO paired_devices (device_id, name, token_hash, paired_at, pulled_seq)
         VALUES (?, ?, ?, ?, 0)",
    )
    .bind(device_id)
    .bind(name)
    .bind(token_hash)
    .bind(now_ms())
    .execute(&db.pool)
    .await?;
    Ok(())
}

pub async fn get(db: &Db, device_id: &str) -> AppResult<Option<PairedDevice>> {
    Ok(
        sqlx::query_as("SELECT * FROM paired_devices WHERE device_id = ?")
            .bind(device_id)
            .fetch_optional(&db.pool)
            .await?,
    )
}

pub async fn list(db: &Db) -> AppResult<Vec<PairedDevice>> {
    Ok(
        sqlx::query_as("SELECT * FROM paired_devices ORDER BY paired_at")
            .fetch_all(&db.pool)
            .await?,
    )
}

/// Removes the device; its next request is refused.
pub async fn revoke(db: &Db, device_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM paired_devices WHERE device_id = ?")
        .bind(device_id)
        .execute(&db.pool)
        .await?;
    Ok(())
}

/// Notes a finished round; `acknowledged` is the log position the device says it has applied.
pub async fn record_sync(db: &Db, device_id: &str, acknowledged: Option<i64>) -> AppResult<()> {
    sqlx::query(
        "UPDATE paired_devices SET last_sync_at = ?, pulled_seq = MAX(pulled_seq, ?)
         WHERE device_id = ?",
    )
    .bind(now_ms())
    .bind(acknowledged.unwrap_or(0))
    .bind(device_id)
    .execute(&db.pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn devices_are_added_replaced_and_revoked() {
        let db = Db::in_memory().await;
        add(&db, "d1", "Phone", "hash-a").await.expect("add");
        record_sync(&db, "d1", Some(7)).await.expect("sync");
        assert_eq!(
            get(&db, "d1").await.expect("get").map(|d| d.pulled_seq),
            Some(7)
        );

        add(&db, "d1", "Phone", "hash-b").await.expect("pair again");
        let again = get(&db, "d1").await.expect("get").expect("device");
        assert_eq!((again.token_hash.as_str(), again.pulled_seq), ("hash-b", 0));

        revoke(&db, "d1").await.expect("revoke");
        assert!(get(&db, "d1").await.expect("get").is_none());
        assert!(list(&db).await.expect("list").is_empty());
    }

    #[tokio::test]
    async fn the_acknowledged_position_never_moves_back() {
        let db = Db::in_memory().await;
        add(&db, "d1", "Phone", "h").await.expect("add");
        record_sync(&db, "d1", Some(9)).await.expect("sync");
        record_sync(&db, "d1", Some(3)).await.expect("sync");
        assert_eq!(
            get(&db, "d1").await.expect("get").map(|d| d.pulled_seq),
            Some(9)
        );
    }
}
