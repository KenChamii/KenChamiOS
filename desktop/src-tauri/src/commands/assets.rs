use crate::db::assets;
use crate::domain::asset::{AssetKind, ProjectAsset, clean_link};
use crate::error::AppResult;
use crate::state::AppState;
use crate::workflow::screenshot_file;
use tauri::State;

#[tauri::command]
pub async fn assets_list(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<ProjectAsset>> {
    assets::list(&state.db, &project_id).await
}

/// Copies the picked image into the app data folder, so the original can move or go.
#[tauri::command]
pub async fn assets_add_screenshot(
    state: State<'_, AppState>,
    project_id: String,
    source_path: String,
    caption: String,
) -> AppResult<ProjectAsset> {
    let imported = screenshot_file::import(&state.assets_dir, &project_id, &source_path)?;
    let caption = if caption.trim().is_empty() {
        imported.stem.clone()
    } else {
        caption
    };
    let added = assets::add(
        &state.db,
        &project_id,
        AssetKind::Screenshot,
        &imported.stored_name,
        &caption,
    )
    .await;
    if added.is_err()
        && let Ok(path) =
            screenshot_file::path_of(&state.assets_dir, &project_id, &imported.stored_name)
    {
        // the row was not saved, so the copy would be orphaned
        let _ = std::fs::remove_file(path);
    }
    added
}

#[tauri::command]
pub async fn assets_add_link(
    state: State<'_, AppState>,
    project_id: String,
    url: String,
    caption: String,
) -> AppResult<ProjectAsset> {
    let url = clean_link(&url)?;
    assets::add(&state.db, &project_id, AssetKind::Link, &url, &caption).await
}

#[tauri::command]
pub async fn assets_set_caption(
    state: State<'_, AppState>,
    id: String,
    caption: String,
) -> AppResult<ProjectAsset> {
    assets::set_caption(&state.db, &id, &caption).await
}

#[tauri::command]
pub async fn assets_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    assets::delete(&state.db, &id).await
}
