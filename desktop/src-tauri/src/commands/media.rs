use crate::db::media;
use crate::domain::media::{EntryInput, LibraryEntry};
use crate::error::AppResult;
use crate::media::covers;
use crate::state::AppState;
use tauri::State;

/// Adds the absolute path of the cached cover, which the ui turns into a file address.
pub fn with_cover(state: &AppState, mut entry: LibraryEntry) -> LibraryEntry {
    entry.cover_path = entry
        .cover_cache_path
        .as_deref()
        .and_then(|name| covers::file_of(&state.covers_dir, name))
        .and_then(|path| path.to_str().map(str::to_string));
    entry
}

#[tauri::command]
pub async fn media_list(state: State<'_, AppState>) -> AppResult<Vec<LibraryEntry>> {
    let entries = media::list(&state.db).await?;
    Ok(entries
        .into_iter()
        .map(|entry| with_cover(&state, entry))
        .collect())
}

#[tauri::command]
pub async fn media_update(
    state: State<'_, AppState>,
    id: String,
    input: EntryInput,
) -> AppResult<LibraryEntry> {
    let entry = media::update(&state.db, &id, input).await?;
    Ok(with_cover(&state, entry))
}

#[tauri::command]
pub async fn media_set_progress(
    state: State<'_, AppState>,
    id: String,
    value: i64,
) -> AppResult<LibraryEntry> {
    let entry = media::set_progress(&state.db, &id, value).await?;
    Ok(with_cover(&state, entry))
}

#[tauri::command]
pub async fn media_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    media::delete(&state.db, &id).await
}
