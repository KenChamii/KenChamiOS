use crate::error::AppResult;
use crate::state::AppState;
use crate::sync::{PairingInfo, SyncView};
use tauri::State;

#[tauri::command]
pub async fn sync_status(state: State<'_, AppState>) -> AppResult<SyncView> {
    state.sync.view().await
}

#[tauri::command]
pub async fn sync_set_enabled(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    state.sync.set_enabled(enabled).await
}

#[tauri::command]
pub async fn sync_pairing_start(state: State<'_, AppState>) -> AppResult<PairingInfo> {
    state.sync.start_pairing().await
}

#[tauri::command]
pub fn sync_pairing_cancel(state: State<'_, AppState>) {
    state.sync.cancel_pairing();
}

#[tauri::command]
pub async fn sync_revoke_device(state: State<'_, AppState>, device_id: String) -> AppResult<()> {
    state.sync.revoke(&device_id).await
}
