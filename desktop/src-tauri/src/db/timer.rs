use super::handle::{Db, now_ms};
use crate::error::AppResult;
use crate::timer::machine::TimerState;
use sqlx::SqliteConnection;

/// The timer saved for this device, if one is running or paused.
pub async fn load(db: &Db) -> AppResult<Option<TimerState>> {
    let state = sqlx::query_as(
        "SELECT project_id, task_id, phase, status, round, phase_started_at,
                phase_length_s, elapsed_ms, running_since
           FROM timer_state WHERE id = ? AND deleted = 0",
    )
    .bind(&db.device_id)
    .fetch_optional(&db.pool)
    .await?;
    Ok(state)
}

// takes a connection so a finished phase and the next state are saved together
pub async fn save(conn: &mut SqliteConnection, db: &Db, state: &TimerState) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO timer_state
            (id, created_at, updated_at, device_id, project_id, task_id, phase, status, round,
             phase_started_at, phase_length_s, elapsed_ms, running_since)
         VALUES (?1, ?2, ?3, ?1, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(id) DO UPDATE SET
            updated_at = excluded.updated_at, deleted = 0, project_id = excluded.project_id,
            task_id = excluded.task_id, phase = excluded.phase, status = excluded.status,
            round = excluded.round, phase_started_at = excluded.phase_started_at,
            phase_length_s = excluded.phase_length_s, elapsed_ms = excluded.elapsed_ms,
            running_since = excluded.running_since",
    )
    .bind(&db.device_id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&state.project_id)
    .bind(&state.task_id)
    .bind(state.phase)
    .bind(state.status)
    .bind(state.round)
    .bind(state.phase_started_at)
    .bind(state.phase_length_s)
    .bind(state.elapsed_ms)
    .bind(state.running_since)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn clear(db: &Db) -> AppResult<()> {
    sqlx::query("UPDATE timer_state SET deleted = 1, updated_at = ? WHERE id = ?")
        .bind(db.stamp())
        .bind(&db.device_id)
        .execute(&db.pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{create, create_input};
    use crate::timer::machine::{Status, TimerConfig};

    const CONFIG: TimerConfig = TimerConfig {
        focus_s: 1500,
        break_s: 300,
        long_break_s: 900,
    };

    async fn state_for_a_task(db: &Db) -> TimerState {
        let project = create_for_test(db, "Demo").await;
        let task = create(db, create_input(&project.id, "Do it"))
            .await
            .expect("task");
        TimerState::start(project.id, task.id, CONFIG, 1_000)
    }

    #[tokio::test]
    async fn saved_state_loads_back_and_updates_in_place() {
        let db = Db::in_memory().await;
        let mut state = state_for_a_task(&db).await;
        assert_eq!(load(&db).await.expect("load"), None);

        let mut conn = db.pool.acquire().await.expect("conn");
        save(&mut conn, &db, &state).await.expect("save");
        state.pause(61_000);
        save(&mut conn, &db, &state).await.expect("save again");
        drop(conn);

        let loaded = load(&db).await.expect("load").expect("saved");
        assert_eq!(loaded, state);
        assert_eq!(loaded.status, Status::Paused);
        assert_eq!(loaded.elapsed_ms, 60_000);
    }

    #[tokio::test]
    async fn cleared_state_is_gone_and_can_be_saved_again() {
        let db = Db::in_memory().await;
        let state = state_for_a_task(&db).await;
        let mut conn = db.pool.acquire().await.expect("conn");
        save(&mut conn, &db, &state).await.expect("save");
        drop(conn);

        clear(&db).await.expect("clear");
        assert_eq!(load(&db).await.expect("load"), None);

        let mut conn = db.pool.acquire().await.expect("conn");
        save(&mut conn, &db, &state).await.expect("save");
        drop(conn);
        assert_eq!(load(&db).await.expect("load"), Some(state));
    }
}
