use crate::db::time_entries;
use crate::domain::time_entry::{TimeEntry, TimeEntryInput};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn time_entries_list(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<TimeEntry>> {
    time_entries::list(&state.db, &project_id).await
}

#[tauri::command]
pub async fn time_entries_create(
    state: State<'_, AppState>,
    project_id: String,
    input: TimeEntryInput,
) -> AppResult<TimeEntry> {
    time_entries::create_manual(&state.db, &project_id, input).await
}

#[tauri::command]
pub async fn time_entries_update(
    state: State<'_, AppState>,
    id: String,
    input: TimeEntryInput,
) -> AppResult<TimeEntry> {
    time_entries::update(&state.db, &id, input).await
}

#[tauri::command]
pub async fn time_entries_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    time_entries::delete(&state.db, &id).await
}
