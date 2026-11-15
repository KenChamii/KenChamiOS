use crate::agent::plan;
use crate::db::{docs, projects, tasks};
use crate::domain::agent_plan::PlanProposal;
use crate::domain::{DocKind, Task};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use tauri::State;

/// Asks Claude to split the project's plan doc into phases and tasks. Creates nothing.
#[tauri::command]
pub async fn agent_plan_preview(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<PlanProposal> {
    let project = projects::get(&state.db, &project_id).await?;
    let plan_md = docs::get(&state.db, &project_id, DocKind::Plan)
        .await?
        .map(|doc| doc.body_md)
        .unwrap_or_default();
    if plan_md.trim().is_empty() {
        return Err(AppError::Invalid("write the plan first"));
    }
    // runs in the app data folder, so no project file can reach the prompt
    let cwd = state.data_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        plan::breakdown(&cwd, &project.project.name, &plan_md)
    })
    .await
    .map_err(|e| AppError::Agent(e.to_string()))?
}

/// Creates the tasks of a plan the user approved (and may have edited), all or none.
#[tauri::command]
pub async fn agent_plan_create(
    state: State<'_, AppState>,
    project_id: String,
    proposal: PlanProposal,
) -> AppResult<Vec<Task>> {
    projects::get(&state.db, &project_id).await?;
    let creates = proposal.clean()?.to_task_creates(&project_id);
    tasks::create_many(&state.db, &creates).await
}
