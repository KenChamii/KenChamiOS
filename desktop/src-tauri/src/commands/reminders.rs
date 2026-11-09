use crate::db::{now_ms, reminders};
use crate::domain::idle::IdleProject;
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn reminders_idle_projects(state: State<'_, AppState>) -> AppResult<Vec<IdleProject>> {
    reminders::idle(&state.db, now_ms()).await
}

#[tauri::command]
pub async fn reminders_snooze(state: State<'_, AppState>, project_id: String) -> AppResult<()> {
    reminders::snooze(&state.db, &project_id, now_ms()).await
}

#[tauri::command]
pub async fn reminders_idle_days_get(state: State<'_, AppState>) -> AppResult<i64> {
    reminders::idle_days(&state.db).await
}

#[tauri::command]
pub async fn reminders_idle_days_set(state: State<'_, AppState>, days: i64) -> AppResult<i64> {
    reminders::set_idle_days(&state.db, days).await
}
