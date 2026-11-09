use super::media::with_cover;
use crate::db::media::{self, ItemOrigin};
use crate::domain::media::{LibraryEntry, ManualInput, MediaSource, MediaStatus, MediaType};
use crate::error::AppResult;
use crate::media::provider::Metadata;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn media_search(
    state: State<'_, AppState>,
    query: String,
    media_type: MediaType,
) -> AppResult<Vec<Metadata>> {
    state.metadata.search(&state.db, &query, media_type).await
}

/// Adds a title found by a search; the details come from the cache when they were seen before.
#[tauri::command]
pub async fn media_add(
    state: State<'_, AppState>,
    media_type: MediaType,
    external_id: String,
    status: MediaStatus,
) -> AppResult<LibraryEntry> {
    let found = state
        .metadata
        .details(&state.db, media_type, &external_id)
        .await?;
    let origin = ItemOrigin {
        media_type,
        source: media_type.source(),
        external_id: Some(&found.external_id),
    };
    let cover_url = found.details.cover_url.clone();
    let entry = media::add(&state.db, origin, found.details, status).await?;
    let Some(url) = cover_url else {
        return Ok(entry);
    };
    // the cover is a nicety, so a failed download must not undo the add
    let downloaded = state
        .metadata
        .download_cover(&state.covers_dir, &entry.media_item_id, &url)
        .await;
    if let Ok(name) = downloaded {
        media::set_cover(&state.db, &entry.media_item_id, &name).await?;
        return Ok(with_cover(&state, media::get(&state.db, &entry.id).await?));
    }
    Ok(with_cover(&state, entry))
}

#[tauri::command]
pub async fn media_add_manual(
    state: State<'_, AppState>,
    input: ManualInput,
) -> AppResult<LibraryEntry> {
    let origin = ItemOrigin {
        media_type: input.media_type,
        source: MediaSource::Manual,
        external_id: None,
    };
    let entry = media::add(&state.db, origin, input.details, input.status).await?;
    Ok(with_cover(&state, entry))
}

#[tauri::command]
pub fn games_key_status(state: State<'_, AppState>) -> AppResult<bool> {
    state.metadata.has_games_key()
}

#[tauri::command]
pub fn games_key_set(state: State<'_, AppState>, key: String) -> AppResult<()> {
    state.metadata.set_games_key(&key)
}

#[tauri::command]
pub fn games_key_clear(state: State<'_, AppState>) -> AppResult<()> {
    state.metadata.clear_games_key()
}

#[tauri::command]
pub fn tmdb_token_status(state: State<'_, AppState>) -> AppResult<bool> {
    state.metadata.has_tmdb_token()
}

#[tauri::command]
pub fn tmdb_token_set(state: State<'_, AppState>, token: String) -> AppResult<()> {
    state.metadata.set_tmdb_token(&token)
}

#[tauri::command]
pub fn tmdb_token_clear(state: State<'_, AppState>) -> AppResult<()> {
    state.metadata.clear_tmdb_token()
}
