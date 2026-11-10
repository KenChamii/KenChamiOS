use crate::db::planner;
use crate::domain::Task;
use crate::domain::planner::{FocusBlock, PlannedTask, TaskPlan};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

/// Sets when a task is planned; the ui computes days and block starts in local time.
#[tauri::command]
pub async fn planner_set_plan(
    state: State<'_, AppState>,
    id: String,
    plan: TaskPlan,
) -> AppResult<Task> {
    planner::set_plan(&state.db, &id, plan).await
}

#[tauri::command]
pub async fn planner_planned(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<PlannedTask>> {
    planner::planned(&state.db, &from, &to).await
}

/// Suggestions for the morning plan; `until` is the end of today in local time.
#[tauri::command]
pub async fn planner_candidates(
    state: State<'_, AppState>,
    today: String,
    until: i64,
    query: Option<String>,
) -> AppResult<Vec<PlannedTask>> {
    planner::candidates(&state.db, &today, until, query.as_deref()).await
}

#[tauri::command]
pub async fn planner_completed(
    state: State<'_, AppState>,
    from: i64,
    to: i64,
) -> AppResult<Vec<PlannedTask>> {
    planner::completed(&state.db, from, to).await
}

#[tauri::command]
pub async fn planner_focus_blocks(
    state: State<'_, AppState>,
    from: i64,
    to: i64,
) -> AppResult<Vec<FocusBlock>> {
    planner::focus_blocks(&state.db, from, to).await
}

#[tauri::command]
pub async fn planner_available_get(state: State<'_, AppState>) -> AppResult<i64> {
    planner::available_minutes(&state.db).await
}

#[tauri::command]
pub async fn planner_available_set(state: State<'_, AppState>, minutes: i64) -> AppResult<i64> {
    planner::set_available_minutes(&state.db, minutes).await
}
