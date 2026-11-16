use crate::db::jobs::{self, JobStats};
use crate::db::now_ms;
use crate::domain::job::{JobApplication, JobInput, JobStage, JobView};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn jobs_list(state: State<'_, AppState>) -> AppResult<Vec<JobView>> {
    jobs::list(&state.db, now_ms()).await
}

#[tauri::command]
pub async fn jobs_create(state: State<'_, AppState>, input: JobInput) -> AppResult<JobApplication> {
    jobs::create(&state.db, input).await
}

#[tauri::command]
pub async fn jobs_update(
    state: State<'_, AppState>,
    id: String,
    input: JobInput,
) -> AppResult<JobApplication> {
    jobs::update(&state.db, &id, input).await
}

/// `today` is the local day, which the ui knows and the core does not.
#[tauri::command]
pub async fn jobs_move(
    state: State<'_, AppState>,
    id: String,
    stage: JobStage,
    today: String,
) -> AppResult<JobApplication> {
    jobs::move_stage(&state.db, &id, stage, &today).await
}

#[tauri::command]
pub async fn jobs_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    jobs::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn jobs_stats(state: State<'_, AppState>) -> AppResult<JobStats> {
    jobs::stats(&state.db, now_ms()).await
}

/// Applications that are due or stale, for the dashboard.
#[tauri::command]
pub async fn jobs_attention(state: State<'_, AppState>) -> AppResult<Vec<JobView>> {
    let all = jobs::list(&state.db, now_ms()).await?;
    Ok(all.into_iter().filter(|v| v.due || v.stale).collect())
}
