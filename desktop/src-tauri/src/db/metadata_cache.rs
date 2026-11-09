use super::handle::Db;
use crate::error::AppResult;

/// A cached answer and when it was fetched.
pub struct Cached {
    pub payload: String,
    pub fetched_at: i64,
}

pub async fn get(db: &Db, key: &str) -> AppResult<Option<Cached>> {
    let row: Option<(String, i64)> =
        sqlx::query_as("SELECT payload, fetched_at FROM metadata_cache WHERE cache_key = ?")
            .bind(key)
            .fetch_optional(&db.pool)
            .await?;
    Ok(row.map(|(payload, fetched_at)| Cached {
        payload,
        fetched_at,
    }))
}

pub async fn put(db: &Db, key: &str, payload: &str, now: i64) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO metadata_cache (cache_key, payload, fetched_at) VALUES (?, ?, ?)
         ON CONFLICT(cache_key) DO UPDATE
            SET payload = excluded.payload, fetched_at = excluded.fetched_at",
    )
    .bind(key)
    .bind(payload)
    .bind(now)
    .execute(&db.pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn answers_are_stored_and_replaced() {
        let db = Db::in_memory().await;
        assert!(get(&db, "k").await.expect("get").is_none());

        put(&db, "k", "one", 10).await.expect("put");
        put(&db, "k", "two", 20).await.expect("replace");

        let cached = get(&db, "k").await.expect("get").expect("hit");
        assert_eq!(cached.payload, "two");
        assert_eq!(cached.fetched_at, 20);
    }
}
