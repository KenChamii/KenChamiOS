use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use crate::domain::civil_date;
use crate::domain::habit::{HabitView, summarize};
use crate::error::{AppError, AppResult};
use std::collections::HashMap;

/// Live habits in the order they were added, summarized for `today` (`YYYY-MM-DD`, local).
pub async fn list(db: &Db, today: &str) -> AppResult<Vec<HabitView>> {
    let today = clean_day(today)?;
    let habits: Vec<(String, String)> =
        sqlx::query_as("SELECT id, name FROM habits WHERE deleted = 0 ORDER BY created_at, id")
            .fetch_all(&db.pool)
            .await?;
    let checks: Vec<(String, String)> =
        sqlx::query_as("SELECT habit_id, day FROM habit_checks WHERE deleted = 0")
            .fetch_all(&db.pool)
            .await?;
    let mut days: HashMap<String, Vec<String>> = HashMap::new();
    for (habit_id, day) in checks {
        days.entry(habit_id).or_default().push(day);
    }
    Ok(habits
        .into_iter()
        .filter_map(|(id, name)| {
            let checked = days.remove(&id).unwrap_or_default();
            summarize(id, name, &checked, &today)
        })
        .collect())
}

pub async fn create(db: &Db, name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("habit name is required"));
    }
    let id = new_id();
    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO habits (id, created_at, updated_at, device_id, name) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(name)
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "habit.created",
        "habit",
        &id,
        &format!("Started habit {name}"),
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// Soft deletes the habit and its check-ins.
pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE habits SET deleted = 1, updated_at = ?, device_id = ? WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "habit")?;
    sqlx::query(
        "UPDATE habit_checks SET deleted = 1, updated_at = ?, device_id = ?
          WHERE habit_id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    activity::record(&mut tx, db, "habit.deleted", "habit", id, "Deleted habit").await?;
    tx.commit().await?;
    Ok(())
}

/// Checks the habit for `day`, or clears the check when it is already there; returns the new state.
pub async fn toggle(db: &Db, habit_id: &str, day: &str) -> AppResult<bool> {
    let day = clean_day(day)?;
    let mut tx = db.pool.begin().await?;
    let name: String = sqlx::query_scalar("SELECT name FROM habits WHERE id = ? AND deleted = 0")
        .bind(habit_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("habit"))?;
    let cleared = sqlx::query(
        "UPDATE habit_checks SET deleted = 1, updated_at = ?, device_id = ?
          WHERE habit_id = ? AND day = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(habit_id)
    .bind(&day)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        > 0;
    if !cleared {
        sqlx::query(
            "INSERT INTO habit_checks (id, created_at, updated_at, device_id, habit_id, day)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id())
        .bind(now_ms())
        .bind(db.stamp())
        .bind(&db.device_id)
        .bind(habit_id)
        .bind(&day)
        .execute(&mut *tx)
        .await?;
    }
    let (kind, summary) = if cleared {
        ("habit.unchecked", format!("Unchecked {name} for {day}"))
    } else {
        ("habit.checked", format!("Checked {name} for {day}"))
    };
    activity::record(&mut tx, db, kind, "habit", habit_id, &summary).await?;
    tx.commit().await?;
    Ok(!cleared)
}

// stored as the canonical YYYY-MM-DD so the unique key matches across devices
fn clean_day(day: &str) -> AppResult<String> {
    civil_date::parse(day)
        .map(civil_date::format)
        .ok_or(AppError::Invalid("day must be a date like 2026-11-10"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;

    #[tokio::test]
    async fn checking_in_builds_a_streak_and_toggling_clears_it() {
        let db = Db::in_memory().await;
        let id = create(&db, " Read ").await.expect("create");

        for day in ["2026-11-08", "2026-11-09"] {
            assert!(toggle(&db, &id, day).await.expect("check"));
        }
        let habits = list(&db, "2026-11-10").await.expect("list");
        assert_eq!(habits[0].name, "Read");
        assert_eq!((habits[0].streak, habits[0].checked_today), (2, false));

        assert!(toggle(&db, &id, "2026-11-10").await.expect("today"));
        assert_eq!(list(&db, "2026-11-10").await.expect("list")[0].streak, 3);

        assert!(!toggle(&db, &id, "2026-11-10").await.expect("undo"));
        let habit = &list(&db, "2026-11-10").await.expect("list")[0];
        assert_eq!((habit.streak, habit.checked_today), (2, false));
        assert_eq!(
            kinds_for(&db, &id).await,
            [
                "habit.created",
                "habit.checked",
                "habit.checked",
                "habit.checked",
                "habit.unchecked"
            ]
        );
    }

    #[tokio::test]
    async fn bad_input_and_deleted_habits_are_refused() {
        let db = Db::in_memory().await;
        assert!(matches!(create(&db, "  ").await, Err(AppError::Invalid(_))));
        let id = create(&db, "Run").await.expect("create");
        assert!(matches!(
            toggle(&db, &id, "2026-02-30").await,
            Err(AppError::Invalid(_))
        ));
        toggle(&db, &id, "2026-11-10").await.expect("check");

        delete(&db, &id).await.expect("delete");

        assert!(list(&db, "2026-11-10").await.expect("list").is_empty());
        assert!(matches!(
            toggle(&db, &id, "2026-11-10").await,
            Err(AppError::NotFound(_))
        ));
        let live_checks: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM habit_checks WHERE deleted = 0")
                .fetch_one(&db.pool)
                .await
                .expect("count");
        assert_eq!(live_checks, 0);
    }
}
