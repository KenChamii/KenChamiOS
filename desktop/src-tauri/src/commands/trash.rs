use crate::db::trash;
use crate::domain::trash::{TrashItem, TrashKind};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn trash_list(state: State<'_, AppState>) -> AppResult<Vec<TrashItem>> {
    trash::list(&state.db).await
}

#[tauri::command]
pub async fn trash_restore(
    state: State<'_, AppState>,
    kind: TrashKind,
    id: String,
) -> AppResult<()> {
    trash::restore(&state.db, kind, &id).await
}
