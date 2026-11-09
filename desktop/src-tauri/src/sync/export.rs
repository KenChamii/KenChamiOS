use super::schema::SyncSchema;
use super::types::{Change, SyncError};
use crate::db::Db;
use serde_json::{Map, Value};
use sqlx::AssertSqlSafe;

pub struct Page {
    pub changes: Vec<Change>,
    pub next: i64,
    pub has_more: bool,
}

/// The rows changed after `since`, current content, up to `limit` log entries.
/// Rows last written by `requester` are skipped: that device already has them.
pub async fn read_page(
    db: &Db,
    schema: &SyncSchema,
    since: i64,
    limit: usize,
    requester: &str,
) -> Result<Page, SyncError> {
    let entries: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT seq, table_name, row_id FROM change_log WHERE seq > ? ORDER BY seq LIMIT ?",
    )
    .bind(since)
    .bind(limit as i64 + 1)
    .fetch_all(&db.pool)
    .await?;
    let has_more = entries.len() > limit;
    let taken = &entries[..entries.len().min(limit)];
    let next = taken.last().map_or(since, |entry| entry.0);

    let mut changes = Vec::new();
    for (_, table_name, row_id) in taken {
        let Some(table) = schema.table(table_name) else {
            continue;
        };
        let json: Option<String> = sqlx::query_scalar(AssertSqlSafe(table.export_sql.as_str()))
            .bind(row_id)
            .fetch_optional(&db.pool)
            .await?;
        let Some(json) = json else { continue };
        let row: Map<String, Value> = serde_json::from_str(&json)
            .map_err(|e| SyncError::App(crate::error::AppError::Json(e)))?;
        if row.get("device_id").and_then(Value::as_str) == Some(requester) {
            continue;
        }
        changes.push(Change {
            table: table.name.clone(),
            row,
        });
    }
    Ok(Page {
        changes,
        next,
        has_more,
    })
}

/// The newest sequence number in the log, 0 when it is empty.
pub async fn latest_seq(db: &Db) -> Result<i64, SyncError> {
    let seq: Option<i64> = sqlx::query_scalar("SELECT MAX(seq) FROM change_log")
        .fetch_one(&db.pool)
        .await?;
    Ok(seq.unwrap_or(0))
}
