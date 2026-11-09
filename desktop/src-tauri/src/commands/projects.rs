use crate::db::projects;
use crate::domain::{ProjectFilter, ProjectInput, ProjectSummary, Stage};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn projects_list(
    state: State<'_, AppState>,
    filter: ProjectFilter,
) -> AppResult<Vec<ProjectSummary>> {
    projects::list(&state.db, &filter).await
}

#[tauri::command]
pub async fn projects_get(state: State<'_, AppState>, id: String) -> AppResult<ProjectSummary> {
    projects::get(&state.db, &id).await
}

#[tauri::command]
pub async fn projects_create(
    state: State<'_, AppState>,
    input: ProjectInput,
) -> AppResult<ProjectSummary> {
    projects::create(&state.db, input).await
}

#[tauri::command]
pub async fn projects_update(
    state: State<'_, AppState>,
    id: String,
    input: ProjectInput,
) -> AppResult<ProjectSummary> {
    projects::update(&state.db, &id, input).await
}

#[tauri::command]
pub async fn projects_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    projects::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn project_set_stage(
    state: State<'_, AppState>,
    id: String,
    stage: Stage,
) -> AppResult<ProjectSummary> {
    projects::set_stage(&state.db, &id, stage).await
}
