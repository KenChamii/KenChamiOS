use crate::db::search;
use crate::domain::search::SearchHit;
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn search_all(state: State<'_, AppState>, query: String) -> AppResult<Vec<SearchHit>> {
    search::search(&state.db, &query).await
}
