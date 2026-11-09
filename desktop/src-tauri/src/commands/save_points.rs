use crate::db::{docs, projects, save_points};
use crate::domain::export::project_markdown;
use crate::domain::{DocKind, SavePoint, SavePointInput};
use crate::error::AppResult;
use crate::git;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn save_points_list(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<SavePoint>> {
    save_points::list(&state.db, &project_id).await
}

// branch and commit come from the repo at save time, not from the ui
#[tauri::command]
pub async fn save_points_create(
    state: State<'_, AppState>,
    project_id: String,
    input: SavePointInput,
) -> AppResult<SavePoint> {
    let project = projects::get(&state.db, &project_id).await?;
    let info = git::read(project.project.repo_path.as_deref())?;
    save_points::create(&state.db, &project_id, input, info.branch(), info.head()).await
}

#[tauri::command]
pub async fn save_points_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    save_points::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn save_points_export(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
) -> AppResult<()> {
    let project = projects::get(&state.db, &project_id).await?;
    let plan = docs::get(&state.db, &project_id, DocKind::Plan).await?;
    let points = save_points::list(&state.db, &project_id).await?;
    std::fs::write(
        path,
        project_markdown(&project.project, plan.as_ref(), &points),
    )?;
    Ok(())
}
