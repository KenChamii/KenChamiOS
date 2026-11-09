use crate::db::{projects, session_notes, tasks};
use crate::domain::session_note::{self, ParsedNote, SessionConfirm};
use crate::domain::{SavePoint, TaskStatus};
use crate::error::AppResult;
use crate::git;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct DoneMatch {
    pub item: String,
    pub task_id: Option<String>,
}

#[derive(Serialize)]
pub struct NotePreview {
    pub parsed: ParsedNote,
    pub done_matches: Vec<DoneMatch>,
}

/// Parses the pasted text and suggests which open tasks the done items belong to.
#[tauri::command]
pub async fn session_note_preview(
    state: State<'_, AppState>,
    project_id: String,
    text: String,
) -> AppResult<NotePreview> {
    projects::get(&state.db, &project_id).await?;
    let parsed = session_note::parse(&text);
    let open: Vec<_> = tasks::list(&state.db, &project_id)
        .await?
        .into_iter()
        .filter(|task| task.status != TaskStatus::Done)
        .collect();
    let done_matches = parsed
        .done
        .iter()
        .map(|item| DoneMatch {
            item: item.clone(),
            task_id: session_note::best_match(item, &open).map(|task| task.id.clone()),
        })
        .collect();
    Ok(NotePreview {
        parsed,
        done_matches,
    })
}

// branch and commit come from the repo at save time, not from the ui
#[tauri::command]
pub async fn session_note_confirm(
    state: State<'_, AppState>,
    input: SessionConfirm,
) -> AppResult<SavePoint> {
    let project = projects::get(&state.db, &input.project_id).await?;
    let info = git::read(project.project.repo_path.as_deref())?;
    session_notes::confirm(&state.db, &input, info.branch(), info.head()).await
}
