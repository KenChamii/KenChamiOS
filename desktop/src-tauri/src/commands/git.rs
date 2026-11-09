use crate::db::projects;
use crate::error::AppResult;
use crate::git::{self, GitInfo};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn git_info(state: State<'_, AppState>, project_id: String) -> AppResult<GitInfo> {
    let project = projects::get(&state.db, &project_id).await?;
    git::read(project.project.repo_path.as_deref())
}
