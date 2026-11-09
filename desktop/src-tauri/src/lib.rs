mod commands;
mod db;
mod domain;
mod error;
mod git;
mod state;
pub mod sync;
mod workflow;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let state = tauri::async_runtime::block_on(AppState::init(data_dir))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info::app_info,
            commands::projects::projects_list,
            commands::projects::projects_get,
            commands::projects::projects_create,
            commands::projects::projects_update,
            commands::projects::projects_delete,
            commands::projects::project_set_stage,
            commands::docs::docs_get,
            commands::docs::docs_upsert,
            commands::tasks::tasks_list,
            commands::tasks::tasks_create,
            commands::tasks::tasks_update,
            commands::tasks::tasks_delete,
            commands::tasks::tasks_move,
            commands::checklist::checklist_list,
            commands::checklist::checklist_create,
            commands::checklist::checklist_update,
            commands::checklist::checklist_delete,
            commands::git::git_info,
            commands::save_points::save_points_list,
            commands::save_points::save_points_create,
            commands::save_points::save_points_delete,
            commands::save_points::save_points_export,
            commands::templates::templates_list,
            commands::templates::templates_update,
            commands::templates::templates_reset,
            commands::templates::templates_preview,
            commands::brief::brief_targets,
            commands::brief::brief_get_target,
            commands::brief::brief_set_target,
            commands::brief::brief_render,
            commands::brief::brief_preview,
            commands::brief::brief_write,
            commands::prompts::prompts_list,
            commands::prompts::prompts_create,
            commands::prompts::prompts_update,
            commands::prompts::prompts_set_favorite,
            commands::prompts::prompts_delete,
            commands::prompts::prompts_variables,
            commands::prompts::prompts_copy,
            commands::prompts::tasks_prompt,
            commands::session_notes::session_note_preview,
            commands::session_notes::session_note_confirm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
