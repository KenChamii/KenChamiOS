use crate::db::habits;
use crate::domain::habit::{HabitCheck, HabitView};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn habits_list(state: State<'_, AppState>, today: String) -> AppResult<Vec<HabitView>> {
    habits::list(&state.db, &today).await
}

#[tauri::command]
pub async fn habits_create(state: State<'_, AppState>, name: String) -> AppResult<String> {
    habits::create(&state.db, &name).await
}

#[tauri::command]
pub async fn habits_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    habits::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn habits_toggle(state: State<'_, AppState>, id: String, day: String) -> AppResult<bool> {
    habits::toggle(&state.db, &id, &day).await
}

/// Check-ins from `from` to `to` (local days, both included), for the weekly review.
#[tauri::command]
pub async fn habits_checks(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<HabitCheck>> {
    habits::checks_between(&state.db, &from, &to).await
}
