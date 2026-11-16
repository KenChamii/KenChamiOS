use super::handle::{Db, now_ms};
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct McpCall {
    pub id: i64,
    pub at: i64,
    pub tool: String,
    pub args: String,
    pub ok: bool,
    pub summary: String,
    /// true while a write can still be reversed
    #[sqlx(rename = "undo")]
    #[serde(skip)]
    pub undo_json: Option<String>,
    pub undone: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallView {
    #[serde(flatten)]
    pub call: McpCall,
    pub can_undo: bool,
}

const MAX_ARGS: usize = 2000;

/// Stores one call. Long arguments are cut, so a huge payload cannot fill the database.
pub async fn record(
    db: &Db,
    tool: &str,
    args: &str,
    ok: bool,
    summary: &str,
    undo: Option<&str>,
) -> AppResult<i64> {
    let args: String = args.chars().take(MAX_ARGS).collect();
    let result = sqlx::query(
        "INSERT INTO mcp_calls (at, tool, args, ok, summary, undo) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(now_ms())
    .bind(tool)
    .bind(args)
    .bind(ok)
    .bind(summary)
    .bind(undo)
    .execute(&db.pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// Newest first.
pub async fn list(db: &Db, limit: i64) -> AppResult<Vec<CallView>> {
    let rows: Vec<McpCall> = sqlx::query_as("SELECT * FROM mcp_calls ORDER BY id DESC LIMIT ?")
        .bind(limit)
        .fetch_all(&db.pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|call| CallView {
            can_undo: call.undo_json.is_some() && !call.undone,
            call,
        })
        .collect())
}

pub async fn get(db: &Db, id: i64) -> AppResult<McpCall> {
    sqlx::query_as("SELECT * FROM mcp_calls WHERE id = ?")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("call"))
}

/// The newest write that was not undone yet.
pub async fn last_undoable(db: &Db) -> AppResult<Option<McpCall>> {
    Ok(sqlx::query_as(
        "SELECT * FROM mcp_calls WHERE undo IS NOT NULL AND undone = 0 ORDER BY id DESC LIMIT 1",
    )
    .fetch_optional(&db.pool)
    .await?)
}

pub async fn mark_undone(db: &Db, id: i64) -> AppResult<()> {
    sqlx::query("UPDATE mcp_calls SET undone = 1 WHERE id = ?")
        .bind(id)
        .execute(&db.pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn calls_are_listed_newest_first_and_undo_is_offered_once() {
        let db = Db::in_memory().await;
        let first = record(&db, "read_today", "{}", true, "ok", None)
            .await
            .expect("first");
        let second = record(&db, "add_task", "{}", true, "added", Some("{}"))
            .await
            .expect("second");

        let list = list(&db, 10).await.expect("list");
        assert_eq!(
            list.iter().map(|c| c.call.id).collect::<Vec<_>>(),
            [second, first]
        );
        assert!(list[0].can_undo && !list[1].can_undo);

        assert_eq!(
            last_undoable(&db).await.expect("last").map(|c| c.id),
            Some(second)
        );
        mark_undone(&db, second).await.expect("undone");
        assert!(last_undoable(&db).await.expect("last").is_none());
        assert!(!get(&db, second).await.expect("get").undo_json.is_none());
    }

    #[tokio::test]
    async fn long_arguments_are_cut() {
        let db = Db::in_memory().await;
        let id = record(&db, "add_task", &"x".repeat(5000), true, "", None)
            .await
            .expect("id");
        assert_eq!(get(&db, id).await.expect("get").args.len(), MAX_ARGS);
    }
}
