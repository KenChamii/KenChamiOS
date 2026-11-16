use crate::db::now_ms;
use crate::db::vault;
use crate::error::AppResult;
use crate::state::AppState;
use crate::vault::sync::{self, Report};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct VaultStatus {
    path: String,
    enabled: bool,
    last_sync: Option<i64>,
    excluded: Vec<String>,
    exists: bool,
}

#[tauri::command]
pub async fn vault_status(state: State<'_, AppState>) -> AppResult<VaultStatus> {
    let path = vault::path(&state.db).await?;
    Ok(VaultStatus {
        exists: std::path::Path::new(&path).is_dir(),
        path,
        enabled: vault::enabled(&state.db).await?,
        last_sync: vault::last_sync(&state.db).await?,
        excluded: vault::excluded(&state.db).await?,
    })
}

#[tauri::command]
pub async fn vault_set_path(state: State<'_, AppState>, path: String) -> AppResult<()> {
    vault::set_path(&state.db, &path).await
}

#[tauri::command]
pub async fn vault_set_enabled(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    vault::set_enabled(&state.db, enabled).await
}

#[tauri::command]
pub async fn vault_set_excluded(state: State<'_, AppState>, names: Vec<String>) -> AppResult<()> {
    vault::set_excluded(&state.db, &names).await
}

/// What a sync would do, without writing anything.
#[tauri::command]
pub async fn vault_preview(state: State<'_, AppState>) -> AppResult<Report> {
    let path = vault::path(&state.db).await?;
    sync::run(&state.db, &state.data_dir, &path, now_ms(), false).await
}

#[tauri::command]
pub async fn vault_sync_now(state: State<'_, AppState>) -> AppResult<Report> {
    sync_once(&state).await
}

pub async fn sync_once(state: &AppState) -> AppResult<Report> {
    let path = vault::path(&state.db).await?;
    sync::run(&state.db, &state.data_dir, &path, now_ms(), true).await
}
