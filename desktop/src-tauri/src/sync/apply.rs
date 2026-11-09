use super::backup;
use super::schema::{SyncSchema, Table};
use super::types::{Change, SyncError};
use super::validate::validate_changes;
use crate::db::{Db, now_ms};
use serde_json::{Map, Value};
use sqlx::query::Query;
use sqlx::sqlite::{Sqlite, SqliteArguments};
use sqlx::{AssertSqlSafe, SqliteConnection};
use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub applied: usize,
    pub ignored: usize,
}

type SqliteQuery<'q> = Query<'q, Sqlite, SqliteArguments>;

fn bind_value<'q>(query: SqliteQuery<'q>, value: &'q Value) -> SqliteQuery<'q> {
    match value {
        Value::String(text) => query.bind(text.as_str()),
        Value::Number(number) => query.bind(number.as_i64()),
        _ => query.bind(None::<i64>),
    }
}

/// Validates a batch and applies it in one transaction, after a backup when a folder is given.
pub async fn apply_changes(
    db: &Db,
    schema: &SyncSchema,
    changes: &[Change],
    backups: Option<&Path>,
) -> Result<ApplyOutcome, SyncError> {
    validate_changes(schema, changes, now_ms())?;
    if changes.is_empty() {
        return Ok(ApplyOutcome::default());
    }
    if let Some(dir) = backups {
        backup::create(&db.pool, dir).await?;
    }
    let mut conn = db.pool.acquire().await?;
    // a page can carry a child before its parent, the next page completes the pair
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await?;
    let result = apply_in_transaction(&mut conn, db, schema, changes).await;
    let restored = sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await;
    let outcome = result?;
    restored?;
    let newest = changes
        .iter()
        .filter_map(|c| c.row.get("updated_at").and_then(Value::as_u64))
        .max()
        .unwrap_or(0);
    db.hlc.merge(newest);
    Ok(outcome)
}

async fn apply_in_transaction(
    conn: &mut SqliteConnection,
    db: &Db,
    schema: &SyncSchema,
    changes: &[Change],
) -> Result<ApplyOutcome, SyncError> {
    let mut tx = sqlx::Connection::begin(conn).await?;
    let mut outcome = ApplyOutcome::default();
    for change in changes {
        let table = schema
            .table(&change.table)
            .ok_or_else(|| SyncError::Rejected("table is not synced".to_string()))?;
        let mut row = change.row.clone();
        resolve_unique_key(&mut tx, db, table, &mut row).await?;
        let mut query = sqlx::query(AssertSqlSafe(table.upsert_sql.as_str()));
        for column in table.shared_columns() {
            query = bind_value(query, &row[&column.name]);
        }
        if query.execute(&mut *tx).await?.rows_affected() > 0 {
            outcome.applied += 1;
        } else {
            outcome.ignored += 1;
        }
    }
    tx.commit().await?;
    Ok(outcome)
}

fn version(row: &Map<String, Value>) -> (i64, &str) {
    (
        row["updated_at"].as_i64().unwrap_or(0),
        row["device_id"].as_str().unwrap_or(""),
    )
}

// two live rows with the same key cannot exist: the newer version keeps the key, the other is deleted
async fn resolve_unique_key(
    conn: &mut SqliteConnection,
    db: &Db,
    table: &Table,
    row: &mut Map<String, Value>,
) -> Result<(), SyncError> {
    let is_live = row["deleted"].as_i64() == Some(0);
    let keys: Vec<&Value> = table.unique_live.iter().map(|c| &row[c]).collect();
    if table.unique_live.is_empty() || !is_live || keys.iter().any(|v| v.is_null()) {
        return Ok(());
    }
    let conditions: Vec<String> = table
        .unique_live
        .iter()
        .map(|c| format!("{c} = ?"))
        .collect();
    let sql = format!(
        "SELECT id, updated_at, device_id FROM {} WHERE deleted = 0 AND id != ? AND {}",
        table.name,
        conditions.join(" AND ")
    );
    let mut query =
        sqlx::query_as::<_, (String, i64, String)>(AssertSqlSafe(sql)).bind(row["id"].as_str());
    for key in keys {
        query = match key {
            Value::String(text) => query.bind(text.as_str()),
            other => query.bind(other.as_i64()),
        };
    }
    let rivals = query.fetch_all(&mut *conn).await?;
    let incoming = version(row);
    let mut incoming_loses = false;
    for (id, updated_at, device_id) in rivals {
        if incoming > (updated_at, device_id.as_str()) {
            sqlx::query(AssertSqlSafe(format!(
                "UPDATE {} SET deleted = 1, updated_at = ?, device_id = ? WHERE id = ?",
                table.name
            )))
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(id)
            .execute(&mut *conn)
            .await?;
        } else {
            incoming_loses = true;
        }
    }
    if incoming_loses {
        row.insert("deleted".to_string(), Value::from(1));
    }
    Ok(())
}
