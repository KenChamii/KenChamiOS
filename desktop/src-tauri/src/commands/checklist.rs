use crate::db::checklist;
use crate::domain::ChecklistItem;
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn checklist_list(
    state: State<'_, AppState>,
    task_id: String,
) -> AppResult<Vec<ChecklistItem>> {
    checklist::list(&state.db, &task_id).await
}

#[tauri::command]
pub async fn checklist_create(
    state: State<'_, AppState>,
    task_id: String,
    text: String,
) -> AppResult<ChecklistItem> {
    checklist::create(&state.db, &task_id, &text).await
}

#[tauri::command]
pub async fn checklist_update(
    state: State<'_, AppState>,
    id: String,
    text: String,
    done: bool,
) -> AppResult<ChecklistItem> {
    checklist::update(&state.db, &id, &text, done).await
}

#[tauri::command]
pub async fn checklist_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    checklist::delete(&state.db, &id).await
}
