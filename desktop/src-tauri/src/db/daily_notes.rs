use super::activity;
use super::handle::{Db, new_id, now_ms};
use crate::domain::civil_date;
use crate::domain::daily_note::{self, DailyNote};
use crate::error::{AppError, AppResult};
use sqlx::SqliteConnection;

/// The note of a local day, `None` while nothing was written for it.
pub async fn get(db: &Db, day: &str) -> AppResult<Option<DailyNote>> {
    let day = civil_date::clean(day)?;
    let mut conn = db.pool.acquire().await?;
    find(&mut conn, &day).await
}

/// Replaces the day's note, creating it on the first save.
pub async fn save(db: &Db, day: &str, body: &str) -> AppResult<DailyNote> {
    let day = civil_date::clean(day)?;
    let mut tx = db.pool.begin().await?;
    let note = write(&mut tx, db, &day, body).await?;
    tx.commit().await?;
    Ok(note)
}

/// Adds `text` at the end of the day's note, after an empty line.
pub async fn append(db: &Db, day: &str, text: &str) -> AppResult<DailyNote> {
    let day = civil_date::clean(day)?;
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Invalid("there is nothing to add to the note"));
    }
    let mut tx = db.pool.begin().await?;
    let current = find(&mut tx, &day)
        .await?
        .map(|n| n.body)
        .unwrap_or_default();
    let note = write(&mut tx, db, &day, &daily_note::append(&current, text)).await?;
    tx.commit().await?;
    Ok(note)
}

/// Days from `from` to `to` (both included) whose note has text, for the calendar dots.
pub async fn days_with_notes(db: &Db, from: &str, to: &str) -> AppResult<Vec<String>> {
    let (from, to) = (civil_date::clean(from)?, civil_date::clean(to)?);
    let days = sqlx::query_scalar(
        "SELECT day FROM daily_notes
          WHERE deleted = 0 AND trim(body) != '' AND day >= ? AND day <= ?
          ORDER BY day",
    )
    .bind(from)
    .bind(to)
    .fetch_all(&db.pool)
    .await?;
    Ok(days)
}

async fn find(conn: &mut SqliteConnection, day: &str) -> AppResult<Option<DailyNote>> {
    let note =
        sqlx::query_as("SELECT id, day, body FROM daily_notes WHERE day = ? AND deleted = 0")
            .bind(day)
            .fetch_optional(conn)
            .await?;
    Ok(note)
}

async fn write(
    conn: &mut SqliteConnection,
    db: &Db,
    day: &str,
    body: &str,
) -> AppResult<DailyNote> {
    let id = match find(conn, day).await? {
        Some(note) => {
            sqlx::query(
                "UPDATE daily_notes SET body = ?, updated_at = ?, device_id = ? WHERE id = ?",
            )
            .bind(body)
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(&note.id)
            .execute(&mut *conn)
            .await?;
            note.id
        }
        None => {
            let id = new_id();
            sqlx::query(
                "INSERT INTO daily_notes (id, created_at, updated_at, device_id, day, body)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(now_ms())
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(day)
            .bind(body)
            .execute(&mut *conn)
            .await?;
            activity::record(
                conn,
                db,
                "daily_note.created",
                "daily_note",
                &id,
                &format!("Started the note for {day}"),
            )
            .await?;
            id
        }
    };
    Ok(DailyNote {
        id,
        day: day.to_string(),
        body: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;

    #[tokio::test]
    async fn saving_twice_keeps_one_note_per_day() {
        let db = Db::in_memory().await;
        assert!(get(&db, "2026-11-10").await.expect("empty").is_none());

        let first = save(&db, "2026-11-10", "Morning").await.expect("first");
        let second = save(&db, "2026-11-10", "Morning\nmore")
            .await
            .expect("second");

        assert_eq!(first.id, second.id);
        let stored = get(&db, "2026-11-10").await.expect("get").expect("note");
        assert_eq!(stored.body, "Morning\nmore");
        assert_eq!(kinds_for(&db, &first.id).await, ["daily_note.created"]);
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM daily_notes")
            .fetch_one(&db.pool)
            .await
            .expect("count");
        assert_eq!(rows, 1);
    }

    #[tokio::test]
    async fn append_adds_below_the_existing_text() {
        let db = Db::in_memory().await;
        assert_eq!(
            append(&db, "2026-11-10", " Shipped it ")
                .await
                .expect("new")
                .body,
            "Shipped it"
        );
        assert_eq!(
            append(&db, "2026-11-10", "Tired").await.expect("more").body,
            "Shipped it\n\nTired"
        );
        assert!(matches!(
            append(&db, "2026-11-10", "  ").await,
            Err(AppError::Invalid(_))
        ));
    }

    #[tokio::test]
    async fn days_with_text_in_range_get_a_dot() {
        let db = Db::in_memory().await;
        save(&db, "2026-11-01", "a").await.expect("a");
        save(&db, "2026-11-05", "  ").await.expect("blank");
        save(&db, "2026-11-30", "b").await.expect("b");
        save(&db, "2026-12-01", "c").await.expect("c");

        let days = days_with_notes(&db, "2026-11-01", "2026-11-30")
            .await
            .expect("days");

        assert_eq!(days, ["2026-11-01", "2026-11-30"]);
        assert!(matches!(
            save(&db, "2026-11-31", "x").await,
            Err(AppError::Invalid(_))
        ));
    }
}
