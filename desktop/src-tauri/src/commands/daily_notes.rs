use crate::db::daily_notes;
use crate::domain::daily_note::DailyNote;
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn daily_notes_get(
    state: State<'_, AppState>,
    day: String,
) -> AppResult<Option<DailyNote>> {
    daily_notes::get(&state.db, &day).await
}

#[tauri::command]
pub async fn daily_notes_save(
    state: State<'_, AppState>,
    day: String,
    body: String,
) -> AppResult<DailyNote> {
    daily_notes::save(&state.db, &day, &body).await
}

#[tauri::command]
pub async fn daily_notes_append(
    state: State<'_, AppState>,
    day: String,
    text: String,
) -> AppResult<DailyNote> {
    daily_notes::append(&state.db, &day, &text).await
}

#[tauri::command]
pub async fn daily_notes_days(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<String>> {
    daily_notes::days_with_notes(&state.db, &from, &to).await
}
