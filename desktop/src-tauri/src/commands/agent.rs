use crate::agent::review::{self, RunDiff};
use crate::agent::stream::{self, StreamEvent};
use crate::agent::{plan, run};
use crate::db::{agent_runs, docs, projects, save_points, tasks};
use crate::domain::agent::{AgentRun, AgentRunView, AgentSettings, RunAction, RunStatus};
use crate::domain::agent_plan::PlanProposal;
use crate::domain::{DocKind, SavePointInput, Task, TaskStatus};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};

const MAX_LOG_EVENTS: usize = 2000;

#[derive(Serialize)]
pub struct RunReview {
    diff: RunDiff,
    test_output: Option<String>,
}

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

#[tauri::command]
pub async fn agent_settings_get(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<AgentSettings> {
    agent_runs::get_settings(&state.db, &project_id).await
}

#[tauri::command]
pub async fn agent_settings_set(
    state: State<'_, AppState>,
    project_id: String,
    settings: AgentSettings,
) -> AppResult<AgentSettings> {
    projects::get(&state.db, &project_id).await?;
    agent_runs::set_settings(&state.db, &project_id, settings).await
}

#[tauri::command]
pub async fn agent_runs_list(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<AgentRunView>> {
    agent_runs::list(&state.db, &project_id).await
}

/// Queues a run for the task and starts it as soon as no other run is going.
#[tauri::command]
pub async fn agent_run_start(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> AppResult<AgentRun> {
    let task = tasks::get(&state.db, &task_id).await?;
    let project = projects::get(&state.db, &task.project_id).await?.project;
    if project
        .repo_path
        .as_deref()
        .is_none_or(|path| path.trim().is_empty())
    {
        return Err(AppError::Agent(
            "link a Git repo to the project first".into(),
        ));
    }
    let open = agent_runs::list(&state.db, &task.project_id).await?;
    let has_run = open.iter().any(|view| {
        view.run.task_id == task_id
            && matches!(
                view.run.status,
                RunStatus::Queued | RunStatus::Running | RunStatus::Review
            )
    });
    if has_run {
        return Err(AppError::Invalid("this task already has a run"));
    }
    let created = agent_runs::create(
        &state.db,
        &task.project_id,
        &task_id,
        &run::log_dir(&state.data_dir),
    )
    .await?;
    run::kick(&app);
    Ok(created)
}

/// The events of a finished or running run, read back from its log.
#[tauri::command]
pub async fn agent_run_log(
    state: State<'_, AppState>,
    run_id: String,
) -> AppResult<Vec<StreamEvent>> {
    let found = agent_runs::get(&state.db, &run_id).await?;
    let text = std::fs::read_to_string(&found.log_path).unwrap_or_default();
    let events: Vec<StreamEvent> = text.lines().flat_map(stream::parse_line).collect();
    let skip = events.len().saturating_sub(MAX_LOG_EVENTS);
    Ok(events.into_iter().skip(skip).collect())
}

#[tauri::command]
pub async fn agent_run_review(state: State<'_, AppState>, run_id: String) -> AppResult<RunReview> {
    let found = agent_runs::get(&state.db, &run_id).await?;
    let test_output = std::fs::read_to_string(run::test_output_path(&found.log_path)).ok();
    let diff = tauri::async_runtime::spawn_blocking(move || run::diff_of(&found))
        .await
        .map_err(|e| AppError::Agent(e.to_string()))??;
    Ok(RunReview { diff, test_output })
}

/// Commits what the agent wrote, in the run's own branch. Nothing reaches the user's branch yet.
#[tauri::command]
pub async fn agent_run_approve(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    message: String,
) -> AppResult<AgentRun> {
    let found = agent_runs::get(&state.db, &run_id).await?;
    if found.status != RunStatus::Review {
        return Err(AppError::Invalid("only a run in review can be approved"));
    }
    let (Some(path), Some(base)) = (found.worktree_path.clone(), found.base_commit.clone()) else {
        return Err(AppError::Invalid("the run has no worktree yet"));
    };
    let sha = tauri::async_runtime::spawn_blocking(move || {
        review::approve(Path::new(&path), &base, &message)
    })
    .await
    .map_err(|e| AppError::Agent(e.to_string()))??;
    agent_runs::set_commit(&state.db, &run_id, &sha).await?;
    let _ = app.emit("agent-run-changed", &run_id);
    agent_runs::get(&state.db, &run_id).await
}

/// Merges an approved run into the checked out branch, then saves a point and finishes the task.
#[tauri::command]
pub async fn agent_run_merge(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
) -> AppResult<AgentRun> {
    let found = agent_runs::get(&state.db, &run_id).await?;
    let (RunStatus::Review, Some(_), Some(branch), Some(path)) = (
        found.status,
        found.commit_sha.as_ref(),
        found.branch.clone(),
        found.worktree_path.clone(),
    ) else {
        return Err(AppError::Invalid("approve the changes before merging"));
    };
    let project = projects::get(&state.db, &found.project_id).await?.project;
    let repo_path = project.repo_path.unwrap_or_default();
    let task = tasks::get(&state.db, &found.task_id).await?;

    let (repo, merge_branch) = (repo_path.clone(), branch.clone());
    let merged = tauri::async_runtime::spawn_blocking(move || review::merge(&repo, &merge_branch))
        .await
        .map_err(|e| AppError::Agent(e.to_string()))??;

    let done = agent_runs::apply(&state.db, &run_id, RunAction::Merge, None).await?;
    tasks::move_to(&state.db, &found.task_id, TaskStatus::Done, None, None).await?;
    save_points::create(
        &state.db,
        &found.project_id,
        SavePointInput {
            note: format!("Merged {branch}"),
            done_md: task.title,
            next_md: String::new(),
        },
        found.base_branch.as_deref(),
        Some(&merged),
    )
    .await?;
    review::remove_worktree(&repo_path, &run_id, Path::new(&path), Some(&branch), false)?;
    let _ = app.emit("agent-run-changed", &run_id);
    Ok(done)
}

/// Runs it again in the same worktree with what the user wants different.
#[tauri::command]
pub async fn agent_run_requeue(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    feedback: String,
) -> AppResult<AgentRun> {
    let queued = agent_runs::requeue(&state.db, &run_id, &feedback).await?;
    run::kick(&app);
    Ok(queued)
}

/// Stops the run that is going, or takes a queued one off the queue.
#[tauri::command]
pub async fn agent_run_stop(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
) -> AppResult<()> {
    if state.agent.stop(&run_id) {
        return Ok(());
    }
    discard(&app, &state, &run_id).await.map(|_| ())
}

#[tauri::command]
pub async fn agent_run_discard(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
) -> AppResult<AgentRun> {
    discard(&app, &state, &run_id).await
}

async fn discard(app: &AppHandle, state: &AppState, run_id: &str) -> AppResult<AgentRun> {
    let found = agent_runs::apply(&state.db, run_id, RunAction::Discard, None).await?;
    let project = projects::get(&state.db, &found.project_id).await?.project;
    if let Some(path) = &found.worktree_path {
        review::remove_worktree(
            project.repo_path.as_deref().unwrap_or_default(),
            run_id,
            Path::new(path),
            found.branch.as_deref(),
            true,
        )?;
    }
    tasks::move_to(&state.db, &found.task_id, TaskStatus::Todo, None, None).await?;
    let _ = app.emit("agent-run-changed", run_id);
    Ok(found)
}
