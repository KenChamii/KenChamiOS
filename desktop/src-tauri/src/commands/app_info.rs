use crate::domain::AppInfo;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        device_id: state.db.device_id.clone(),
        db_path: state.db_path.to_string_lossy().into_owned(),
    }
}
