use super::handle::{Db, new_id, now_ms};
use crate::error::AppResult;
use sqlx::SqliteConnection;

/// Adds a timeline row; call it inside the transaction of the write it describes.
pub async fn record(
    conn: &mut SqliteConnection,
    db: &Db,
    kind: &str,
    entity_type: &str,
    entity_id: &str,
    summary: &str,
) -> AppResult<()> {
    let now = now_ms();
    sqlx::query(
        "INSERT INTO activity
            (id, created_at, updated_at, device_id, kind, entity_type, entity_id, summary, at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new_id())
    .bind(now)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(kind)
    .bind(entity_type)
    .bind(entity_id)
    .bind(summary)
    .bind(now)
    .execute(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
pub async fn kinds_for(db: &Db, entity_id: &str) -> Vec<String> {
    sqlx::query_scalar("SELECT kind FROM activity WHERE entity_id = ? ORDER BY at, rowid")
        .bind(entity_id)
        .fetch_all(&db.pool)
        .await
        .expect("read activity")
}
