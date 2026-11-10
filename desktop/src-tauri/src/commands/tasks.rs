use crate::db::tasks;
use crate::domain::task::TaskMoved;
use crate::domain::{Task, TaskCreate, TaskStatus, TaskUpdate};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn tasks_list(state: State<'_, AppState>, project_id: String) -> AppResult<Vec<Task>> {
    tasks::list(&state.db, &project_id).await
}

#[tauri::command]
pub async fn tasks_create(state: State<'_, AppState>, input: TaskCreate) -> AppResult<Task> {
    tasks::create(&state.db, input).await
}

#[tauri::command]
pub async fn tasks_update(
    state: State<'_, AppState>,
    id: String,
    input: TaskUpdate,
) -> AppResult<Task> {
    tasks::update(&state.db, &id, input).await
}

#[tauri::command]
pub async fn tasks_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    tasks::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn tasks_move(
    state: State<'_, AppState>,
    id: String,
    status: TaskStatus,
    before_id: Option<String>,
    next_due_at: Option<i64>,
) -> AppResult<TaskMoved> {
    tasks::move_to(&state.db, &id, status, before_id.as_deref(), next_due_at).await
}

#[tauri::command]
pub async fn tasks_move_to_project(
    state: State<'_, AppState>,
    id: String,
    project_id: String,
) -> AppResult<Task> {
    tasks::move_to_project(&state.db, &id, &project_id).await
}
