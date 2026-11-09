use crate::db::{backups, restore, snapshot};
use crate::domain::backup::BackupStatus;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use std::path::Path;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn backups_status(state: State<'_, AppState>) -> AppResult<BackupStatus> {
    backups::status(&state.db, &state.default_backups_dir).await
}

#[tauri::command]
pub async fn backups_configure(
    state: State<'_, AppState>,
    folder: Option<String>,
    keep: i64,
) -> AppResult<BackupStatus> {
    backups::configure(&state.db, folder.as_deref(), keep).await?;
    backups::status(&state.db, &state.default_backups_dir).await
}

#[tauri::command]
pub async fn backups_run(state: State<'_, AppState>) -> AppResult<BackupStatus> {
    backups::run(&state.db, &state.default_backups_dir).await?;
    backups::status(&state.db, &state.default_backups_dir).await
}

/// Replaces the database with one of the listed backups and restarts the app.
#[tauri::command]
pub async fn backups_restore(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> AppResult<()> {
    let folder = backups::folder(&state.db, &state.default_backups_dir).await?;
    let path = Path::new(&path);
    if !snapshot::list(&folder, backups::PREFIX)?
        .iter()
        .any(|copy| copy == path)
    {
        return Err(AppError::NotFound("backup"));
    }
    restore::stage(&state.data_dir, path)?;
    app.request_restart();
    Ok(())
}
