use crate::db::{prompts, tasks, templates};
use crate::domain::render::{render, unknown_variables};
use crate::domain::{Prompt, PromptInput, TemplateKind};
use crate::error::AppResult;
use crate::state::AppState;
use crate::workflow::context::{prompt_context, task_context};
use std::collections::BTreeMap;
use tauri::State;

#[tauri::command]
pub async fn prompts_list(state: State<'_, AppState>) -> AppResult<Vec<Prompt>> {
    prompts::list(&state.db).await
}

#[tauri::command]
pub async fn prompts_create(state: State<'_, AppState>, input: PromptInput) -> AppResult<Prompt> {
    prompts::create(&state.db, input).await
}

#[tauri::command]
pub async fn prompts_update(
    state: State<'_, AppState>,
    id: String,
    input: PromptInput,
) -> AppResult<Prompt> {
    prompts::update(&state.db, &id, input).await
}

#[tauri::command]
pub async fn prompts_set_favorite(
    state: State<'_, AppState>,
    id: String,
    favorite: bool,
) -> AppResult<Prompt> {
    prompts::set_favorite(&state.db, &id, favorite).await
}

#[tauri::command]
pub async fn prompts_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    prompts::delete(&state.db, &id).await
}

/// Variables the prompt uses that the app cannot fill in itself.
#[tauri::command]
pub async fn prompts_variables(state: State<'_, AppState>, id: String) -> AppResult<Vec<String>> {
    unknown_variables(&prompts::get(&state.db, &id).await?.body_md)
}

/// Renders the prompt for the clipboard and counts the use.
#[tauri::command]
pub async fn prompts_copy(
    state: State<'_, AppState>,
    id: String,
    project_id: Option<String>,
    variables: BTreeMap<String, String>,
) -> AppResult<String> {
    let prompt = prompts::get(&state.db, &id).await?;
    let ctx = prompt_context(&state.db, project_id.as_deref(), variables).await?;
    let text = render(&prompt.body_md, &ctx)?;
    prompts::mark_used(&state.db, &id).await?;
    Ok(text)
}

/// The task prompt template filled for one task.
#[tauri::command]
pub async fn tasks_prompt(state: State<'_, AppState>, task_id: String) -> AppResult<String> {
    tasks::get(&state.db, &task_id).await?;
    let template = templates::get_default(&state.db, TemplateKind::TaskPrompt).await?;
    let ctx = task_context(&state.db, &task_id).await?;
    render(&template.body, &ctx)
}
