use crate::db::now_ms;
use crate::error::AppResult;
use crate::state::AppState;
use crate::timer::config::{self, TimerSettings};
use crate::timer::machine::TimerView;
use crate::timer::shell::publish;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn timer_get(state: State<'_, AppState>) -> AppResult<Option<TimerView>> {
    Ok(state.timer.view(now_ms()).await)
}

#[tauri::command]
pub async fn timer_start(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> AppResult<TimerView> {
    let view = state.timer.start(&state.db, &task_id, now_ms()).await?;
    publish(&app, Some(&view));
    Ok(view)
}

#[tauri::command]
pub async fn timer_pause(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<TimerView>> {
    let view = state.timer.pause(&state.db, now_ms()).await?;
    publish(&app, view.as_ref());
    Ok(view)
}

#[tauri::command]
pub async fn timer_resume(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<TimerView>> {
    let view = state.timer.resume(&state.db, now_ms()).await?;
    publish(&app, view.as_ref());
    Ok(view)
}

#[tauri::command]
pub async fn timer_stop(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    state.timer.stop(&state.db).await?;
    publish(&app, None);
    Ok(())
}

#[tauri::command]
pub async fn timer_settings_get(state: State<'_, AppState>) -> AppResult<TimerSettings> {
    config::load(&state.db).await
}

#[tauri::command]
pub async fn timer_settings_set(
    state: State<'_, AppState>,
    settings: TimerSettings,
) -> AppResult<TimerSettings> {
    config::save(&state.db, settings).await
}
