use crate::db::docs;
use crate::domain::{DocKind, ProjectDoc};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn docs_get(
    state: State<'_, AppState>,
    project_id: String,
    kind: DocKind,
) -> AppResult<Option<ProjectDoc>> {
    docs::get(&state.db, &project_id, kind).await
}

#[tauri::command]
pub async fn docs_upsert(
    state: State<'_, AppState>,
    project_id: String,
    kind: DocKind,
    title: String,
    body_md: String,
) -> AppResult<ProjectDoc> {
    docs::upsert(&state.db, &project_id, kind, &title, &body_md).await
}
