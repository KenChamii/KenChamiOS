use crate::db::media;
use crate::db::now_ms;
use crate::domain::media::{EntryInput, LibraryEntry, MediaSource, MediaStatus, MediaType};
use crate::domain::recommend::{Recommendation, recommend};
use crate::error::AppResult;
use crate::media::covers;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

const RECOMMENDATIONS: usize = 10;
// a TMDB date has no time, so today's episode stays listed until the day is over
const UPCOMING_GRACE_MS: i64 = 24 * 60 * 60 * 1000;

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

/// Planned entries that match the user's own ratings, optionally of one type.
#[tauri::command]
pub async fn media_recommendations(
    state: State<'_, AppState>,
    media_type: Option<MediaType>,
) -> AppResult<Vec<Recommendation>> {
    let entries = media::list(&state.db).await?;
    Ok(recommend(&entries, now_ms(), media_type, RECOMMENDATIONS))
}

#[derive(Serialize)]
pub struct UpcomingRelease {
    entry: LibraryEntry,
    episode: i64,
    air_at_ms: i64,
}

/// Next episodes of the anime and shows the user follows or plans, soonest first.
#[tauri::command]
pub async fn media_upcoming(state: State<'_, AppState>) -> AppResult<Vec<UpcomingRelease>> {
    let followed: Vec<LibraryEntry> = media::list(&state.db)
        .await?
        .into_iter()
        .filter(|entry| matches!(entry.status, MediaStatus::InProgress | MediaStatus::Planned))
        .collect();
    let ids = |type_: MediaType, source: MediaSource| -> Vec<String> {
        followed
            .iter()
            .filter(|entry| entry.media_type == type_ && entry.source == source)
            .filter_map(|entry| entry.external_id.clone())
            .collect()
    };
    let anilist_ids = ids(MediaType::Anime, MediaSource::Anilist);
    let tv_ids = ids(MediaType::Show, MediaSource::Tmdb);
    let airing = state
        .metadata
        .upcoming(&state.db, &anilist_ids, &tv_ids)
        .await?;

    let cutoff = now_ms() - UPCOMING_GRACE_MS;
    let mut releases: Vec<UpcomingRelease> = airing
        .into_iter()
        .filter(|next| next.air_at_ms >= cutoff)
        .filter_map(|next| {
            let entry = followed
                .iter()
                .find(|entry| entry.external_id.as_deref() == Some(next.external_id.as_str()))?;
            Some(UpcomingRelease {
                entry: with_cover(&state, entry.clone()),
                episode: next.episode,
                air_at_ms: next.air_at_ms,
            })
        })
        .collect();
    releases.sort_by_key(|release| release.air_at_ms);
    Ok(releases)
}
