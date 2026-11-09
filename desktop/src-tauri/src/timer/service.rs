use super::config;
use super::machine::{Phase, TimerState, TimerView};
use crate::db::time_entries::{self, NewEntry};
use crate::db::{Db, tasks, timer};
use crate::domain::time_entry::EntryKind;
use crate::error::{AppError, AppResult};
use tauri::async_runtime::Mutex;

/// The one timer of this device. Time is always passed in, so tests need no waiting.
pub struct TimerService {
    state: Mutex<Option<TimerState>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseChange {
    pub finished: Phase,
    pub next: Phase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub view: Option<TimerView>,
    pub change: Option<PhaseChange>,
}

impl TimerService {
    /// Picks up the timer that was running when the app last closed.
    pub async fn recover(db: &Db) -> AppResult<Self> {
        Ok(Self {
            state: Mutex::new(timer::load(db).await?),
        })
    }

    pub async fn view(&self, now: i64) -> Option<TimerView> {
        self.state.lock().await.as_ref().map(|s| s.view(now))
    }

    pub async fn start(&self, db: &Db, task_id: &str, now: i64) -> AppResult<TimerView> {
        let mut guard = self.state.lock().await;
        if guard.is_some() {
            return Err(AppError::Invalid("a timer is already running"));
        }
        let task = tasks::get(db, task_id).await?;
        let config = config::load(db).await?.config();
        let state = TimerState::start(task.project_id, task.id, config, now);
        persist(db, &state).await?;
        let view = state.view(now);
        *guard = Some(state);
        Ok(view)
    }

    pub async fn pause(&self, db: &Db, now: i64) -> AppResult<Option<TimerView>> {
        self.change(db, now, |state| state.pause(now)).await
    }

    pub async fn resume(&self, db: &Db, now: i64) -> AppResult<Option<TimerView>> {
        self.change(db, now, |state| state.resume(now)).await
    }

    /// Drops the timer; the unfinished phase is not logged.
    pub async fn stop(&self, db: &Db) -> AppResult<()> {
        let mut guard = self.state.lock().await;
        timer::clear(db).await?;
        *guard = None;
        Ok(())
    }

    /// Ends the current phase when it is due, logging a finished focus phase.
    pub async fn tick(&self, db: &Db, now: i64) -> AppResult<Tick> {
        let mut guard = self.state.lock().await;
        let Some(state) = guard.as_mut() else {
            return Ok(Tick {
                view: None,
                change: None,
            });
        };
        if !state.is_due(now) {
            return Ok(Tick {
                view: Some(state.view(now)),
                change: None,
            });
        }

        let config = config::load(db).await?.config();
        let mut next = state.clone();
        let Some(finished) = next.advance(config, now) else {
            return Ok(Tick {
                view: Some(state.view(now)),
                change: None,
            });
        };
        let mut tx = db.pool.begin().await?;
        if finished.phase == Phase::Focus {
            time_entries::insert(
                &mut tx,
                db,
                &NewEntry {
                    project_id: &next.project_id,
                    task_id: Some(&next.task_id),
                    kind: EntryKind::Focus,
                    started_at: finished.started_at,
                    ended_at: finished.ended_at,
                    duration_s: i64::from(finished.duration_s),
                    note: "",
                },
            )
            .await?;
        }
        timer::save(&mut tx, db, &next).await?;
        tx.commit().await?;

        let change = PhaseChange {
            finished: finished.phase,
            next: next.phase,
        };
        let view = next.view(now);
        *state = next;
        Ok(Tick {
            view: Some(view),
            change: Some(change),
        })
    }

    async fn change(
        &self,
        db: &Db,
        now: i64,
        apply: impl FnOnce(&mut TimerState),
    ) -> AppResult<Option<TimerView>> {
        let mut guard = self.state.lock().await;
        let Some(state) = guard.as_mut() else {
            return Ok(None);
        };
        let mut updated = state.clone();
        apply(&mut updated);
        persist(db, &updated).await?;
        let view = updated.view(now);
        *state = updated;
        Ok(Some(view))
    }
}

async fn persist(db: &Db, state: &TimerState) -> AppResult<()> {
    let mut conn = db.pool.acquire().await?;
    timer::save(&mut conn, db, state).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{create, create_input};
    use crate::timer::config::TimerSettings;
    use crate::timer::machine::Status;

    const MINUTE: i64 = 60_000;
    const T0: i64 = 1_000_000;

    async fn setup() -> (Db, String, String) {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        let task = create(&db, create_input(&project.id, "Do it"))
            .await
            .expect("task");
        (db, project.id, task.id)
    }

    async fn focus_entries(db: &Db, project_id: &str) -> Vec<(i64, i64, i64)> {
        let mut rows: Vec<_> = time_entries::list(db, project_id)
            .await
            .expect("list")
            .into_iter()
            .filter(|e| e.kind == EntryKind::Focus)
            .map(|e| (e.started_at, e.ended_at, e.duration_s))
            .collect();
        rows.reverse();
        rows
    }

    #[tokio::test]
    async fn start_counts_down_and_rejects_a_second_timer() {
        let (db, _, task_id) = setup().await;
        let service = TimerService::recover(&db).await.expect("recover");
        assert_eq!(service.view(T0).await, None);

        let view = service.start(&db, &task_id, T0).await.expect("start");
        assert_eq!((view.phase, view.remaining_s), (Phase::Focus, 25 * 60));

        let again = service.start(&db, &task_id, T0).await;
        assert!(matches!(again, Err(AppError::Invalid(_))));
        let tick = service.tick(&db, T0 + 10 * MINUTE).await.expect("tick");
        assert_eq!(tick.view.map(|v| v.remaining_s), Some(15 * 60));
        assert_eq!(tick.change, None);
    }

    #[tokio::test]
    async fn finished_focus_logs_an_entry_and_starts_the_break() {
        let (db, project_id, task_id) = setup().await;
        let service = TimerService::recover(&db).await.expect("recover");
        service.start(&db, &task_id, T0).await.expect("start");

        let tick = service.tick(&db, T0 + 25 * MINUTE).await.expect("tick");

        assert_eq!(
            tick.change,
            Some(PhaseChange {
                finished: Phase::Focus,
                next: Phase::Break
            })
        );
        assert_eq!(tick.view.map(|v| v.phase), Some(Phase::Break));
        assert_eq!(
            focus_entries(&db, &project_id).await,
            [(T0, T0 + 25 * MINUTE, 25 * 60)]
        );
        let entry = &time_entries::list(&db, &project_id).await.expect("list")[0];
        assert_eq!(entry.task_id.as_deref(), Some(task_id.as_str()));
    }

    #[tokio::test]
    async fn long_break_follows_the_fourth_focus_and_breaks_are_not_logged() {
        let (db, project_id, task_id) = setup().await;
        let service = TimerService::recover(&db).await.expect("recover");
        service.start(&db, &task_id, T0).await.expect("start");

        let mut now = T0;
        let mut phases = Vec::new();
        for _ in 0..8 {
            let length = service.view(now).await.expect("running").length_s;
            now += i64::from(length) * 1000;
            let tick = service.tick(&db, now).await.expect("tick");
            phases.push(tick.change.expect("phase ended").next);
        }

        assert_eq!(
            phases,
            [
                Phase::Break,
                Phase::Focus,
                Phase::Break,
                Phase::Focus,
                Phase::Break,
                Phase::Focus,
                Phase::LongBreak,
                Phase::Focus
            ]
        );
        assert_eq!(focus_entries(&db, &project_id).await.len(), 4);
        assert_eq!(
            time_entries::list(&db, &project_id)
                .await
                .expect("list")
                .len(),
            4
        );
    }

    #[tokio::test]
    async fn pause_and_resume_move_the_deadline() {
        let (db, project_id, task_id) = setup().await;
        let service = TimerService::recover(&db).await.expect("recover");
        service.start(&db, &task_id, T0).await.expect("start");

        let paused = service
            .pause(&db, T0 + 10 * MINUTE)
            .await
            .expect("pause")
            .expect("active");
        assert_eq!(paused.status, Status::Paused);
        let tick = service.tick(&db, T0 + 90 * MINUTE).await.expect("tick");
        assert_eq!(tick.change, None);
        assert_eq!(tick.view.map(|v| v.remaining_s), Some(15 * 60));

        service
            .resume(&db, T0 + 100 * MINUTE)
            .await
            .expect("resume");
        let early = service.tick(&db, T0 + 114 * MINUTE).await.expect("tick");
        assert_eq!(early.change, None);
        let done = service.tick(&db, T0 + 115 * MINUTE).await.expect("tick");
        assert!(done.change.is_some());
        assert_eq!(focus_entries(&db, &project_id).await.len(), 1);
    }

    #[tokio::test]
    async fn a_restarted_app_continues_the_saved_timer() {
        let (db, project_id, task_id) = setup().await;
        let before = TimerService::recover(&db).await.expect("recover");
        before.start(&db, &task_id, T0).await.expect("start");
        before.pause(&db, T0 + 5 * MINUTE).await.expect("pause");
        before.resume(&db, T0 + 6 * MINUTE).await.expect("resume");
        drop(before);

        let after = TimerService::recover(&db).await.expect("recover");
        let view = after.view(T0 + 16 * MINUTE).await.expect("recovered");
        assert_eq!(view.status, Status::Running);
        assert_eq!(view.remaining_s, 10 * 60);

        // the app stayed closed past the end of the phase
        let tick = after.tick(&db, T0 + 60 * MINUTE).await.expect("tick");
        assert_eq!(tick.change.map(|c| c.finished), Some(Phase::Focus));
        assert_eq!(
            focus_entries(&db, &project_id).await,
            [(T0, T0 + 26 * MINUTE, 25 * 60)]
        );
    }

    #[tokio::test]
    async fn a_paused_timer_is_recovered_paused() {
        let (db, _, task_id) = setup().await;
        let before = TimerService::recover(&db).await.expect("recover");
        before.start(&db, &task_id, T0).await.expect("start");
        before.pause(&db, T0 + 5 * MINUTE).await.expect("pause");

        let after = TimerService::recover(&db).await.expect("recover");

        let view = after.view(T0 + 500 * MINUTE).await.expect("recovered");
        assert_eq!(view.status, Status::Paused);
        assert_eq!(view.remaining_s, 20 * 60);
    }

    #[tokio::test]
    async fn stop_clears_the_timer_for_good() {
        let (db, project_id, task_id) = setup().await;
        let service = TimerService::recover(&db).await.expect("recover");
        service.start(&db, &task_id, T0).await.expect("start");

        service.stop(&db).await.expect("stop");

        assert_eq!(service.view(T0).await, None);
        assert_eq!(
            TimerService::recover(&db)
                .await
                .expect("recover")
                .view(T0)
                .await,
            None
        );
        assert!(focus_entries(&db, &project_id).await.is_empty());
        assert_eq!(service.pause(&db, T0).await.expect("pause"), None);
    }

    #[tokio::test]
    async fn new_phases_use_the_saved_lengths() {
        let (db, _, task_id) = setup().await;
        config::save(
            &db,
            TimerSettings {
                focus_min: 10,
                break_min: 2,
                long_break_min: 20,
            },
        )
        .await
        .expect("save");
        let service = TimerService::recover(&db).await.expect("recover");

        let view = service.start(&db, &task_id, T0).await.expect("start");
        assert_eq!(view.length_s, 10 * 60);

        let tick = service.tick(&db, T0 + 10 * MINUTE).await.expect("tick");
        assert_eq!(tick.view.map(|v| v.length_s), Some(2 * 60));
    }
}
