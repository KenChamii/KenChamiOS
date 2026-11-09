use crate::db::templates;
use crate::domain::ExportTemplate;
use crate::domain::render::{RenderContext, render};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn templates_list(state: State<'_, AppState>) -> AppResult<Vec<ExportTemplate>> {
    templates::list(&state.db).await
}

#[tauri::command]
pub async fn templates_update(
    state: State<'_, AppState>,
    id: String,
    body: String,
) -> AppResult<ExportTemplate> {
    templates::update_body(&state.db, &id, &body).await
}

#[tauri::command]
pub async fn templates_reset(state: State<'_, AppState>, id: String) -> AppResult<ExportTemplate> {
    templates::reset(&state.db, &id).await
}

/// Renders `body` against made-up data so edits can be checked before saving.
#[tauri::command]
pub fn templates_preview(body: String) -> AppResult<String> {
    render(&body, &RenderContext::sample())
}
