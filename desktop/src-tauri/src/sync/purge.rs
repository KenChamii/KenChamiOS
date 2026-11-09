use super::types::SyncError;
use crate::db::Db;
use sqlx::AssertSqlSafe;

const TOMBSTONE_AGE_MS: i64 = 30 * 24 * 60 * 60 * 1000;
const HLC_COUNTER_BITS: u32 = 16;

// children before parents, so most rows can go in one run
pub const PURGE_ORDER: [&str; 11] = [
    "activity",
    "media_progress_log",
    "media_entries",
    "media_items",
    "checklist_items",
    "time_entries",
    "save_points",
    "project_docs",
    "prompts",
    "tasks",
    "projects",
];

/// Hard-deletes tombstones older than 30 days that every paired device has already received.
/// Without a paired device nothing is purged. Returns how many rows were removed.
pub async fn purge_tombstones(db: &Db, now_ms: i64) -> Result<usize, SyncError> {
    let acknowledged: Option<i64> =
        sqlx::query_scalar("SELECT MIN(pulled_seq) FROM paired_devices")
            .fetch_one(&db.pool)
            .await?;
    let Some(acknowledged) = acknowledged else {
        return Ok(0);
    };
    let cutoff = (now_ms - TOMBSTONE_AGE_MS) << HLC_COUNTER_BITS;
    let mut removed = 0;
    for table in PURGE_ORDER {
        let ids: Vec<String> = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT t.id FROM {table} t JOIN change_log l ON l.table_name = ? AND l.row_id = t.id
             WHERE t.deleted = 1 AND t.updated_at < ? AND l.seq <= ?"
        )))
        .bind(table)
        .bind(cutoff)
        .bind(acknowledged)
        .fetch_all(&db.pool)
        .await?;
        for id in ids {
            match sqlx::query(AssertSqlSafe(format!("DELETE FROM {table} WHERE id = ?")))
                .bind(&id)
                .execute(&db.pool)
                .await
            {
                Ok(_) => {
                    sqlx::query("DELETE FROM change_log WHERE table_name = ? AND row_id = ?")
                        .bind(table)
                        .bind(&id)
                        .execute(&db.pool)
                        .await?;
                    removed += 1;
                }
                // a row that is still referenced goes on the next run
                Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(removed)
}
