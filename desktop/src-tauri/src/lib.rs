mod commands;
mod db;
mod domain;
mod error;
mod state;
pub mod sync;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let state = tauri::async_runtime::block_on(AppState::init(data_dir))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::app_info::app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
