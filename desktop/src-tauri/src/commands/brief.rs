use crate::db::{projects, settings, templates};
use crate::domain::TemplateKind;
use crate::domain::brief::{DEFAULT_TARGET, TARGETS, ensure_known_target};
use crate::domain::render::render;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::workflow::brief_file::{self, BriefPreview};
use crate::workflow::context::project_context;
use tauri::State;

fn target_key(project_id: &str) -> String {
    format!("project.{project_id}.brief_target")
}

#[tauri::command]
pub fn brief_targets() -> Vec<&'static str> {
    TARGETS.to_vec()
}

#[tauri::command]
pub async fn brief_get_target(state: State<'_, AppState>, project_id: String) -> AppResult<String> {
    target_of(&state, &project_id).await
}

#[tauri::command]
pub async fn brief_set_target(
    state: State<'_, AppState>,
    project_id: String,
    target: String,
) -> AppResult<()> {
    ensure_known_target(&target)?;
    projects::get(&state.db, &project_id).await?;
    settings::set_string(&state.db, &target_key(&project_id), &target).await
}

/// The rendered brief, for the clipboard.
#[tauri::command]
pub async fn brief_render(state: State<'_, AppState>, project_id: String) -> AppResult<String> {
    render_brief(&state, &project_id).await
}

#[tauri::command]
pub async fn brief_preview(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<BriefPreview> {
    let repo_path = repo_path_of(&state, &project_id).await?;
    let target = target_of(&state, &project_id).await?;
    let rendered = render_brief(&state, &project_id).await?;
    brief_file::preview(&repo_path, &target, &rendered)
}

#[tauri::command]
pub async fn brief_write(state: State<'_, AppState>, project_id: String) -> AppResult<()> {
    let repo_path = repo_path_of(&state, &project_id).await?;
    let target = target_of(&state, &project_id).await?;
    let rendered = render_brief(&state, &project_id).await?;
    brief_file::write(&repo_path, &target, &rendered)
}

async fn render_brief(state: &AppState, project_id: &str) -> AppResult<String> {
    let template = templates::get_default(&state.db, TemplateKind::Brief).await?;
    let ctx = project_context(&state.db, project_id).await?;
    render(&template.body, &ctx)
}

// only the known files are ever used, whatever is stored
async fn target_of(state: &AppState, project_id: &str) -> AppResult<String> {
    let stored = settings::get_string(&state.db, &target_key(project_id)).await?;
    let target = stored.unwrap_or_else(|| DEFAULT_TARGET.to_string());
    ensure_known_target(&target)?;
    Ok(target)
}

async fn repo_path_of(state: &AppState, project_id: &str) -> AppResult<String> {
    projects::get(&state.db, project_id)
        .await?
        .project
        .repo_path
        .filter(|path| !path.trim().is_empty())
        .ok_or(AppError::Invalid(
            "set the repository path of the project first",
        ))
}
