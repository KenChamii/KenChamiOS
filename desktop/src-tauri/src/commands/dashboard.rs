use crate::db::dashboard;
use crate::domain::dashboard::{DueTask, RecapRange, RecapView, TimelineGroup, TimelineItem};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

/// Open tasks due before `until`; the UI passes the end of today in its own time zone.
#[tauri::command]
pub async fn dashboard_due_tasks(
    state: State<'_, AppState>,
    until: i64,
) -> AppResult<Vec<DueTask>> {
    dashboard::due_tasks(&state.db, until).await
}

#[tauri::command]
pub async fn dashboard_timeline(
    state: State<'_, AppState>,
    group: Option<TimelineGroup>,
    limit: i64,
) -> AppResult<Vec<TimelineItem>> {
    dashboard::timeline(&state.db, group, limit).await
}

/// Focus, media and project movement for a period given in the UI's local time boundaries.
#[tauri::command]
pub async fn dashboard_recap(
    state: State<'_, AppState>,
    range: RecapRange,
) -> AppResult<RecapView> {
    dashboard::recap(&state.db, &range).await
}
