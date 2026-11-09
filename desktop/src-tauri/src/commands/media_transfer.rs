use crate::db::{media, media_import};
use crate::domain::media_transfer::{ImportFormat, ImportPreview, ImportSummary, ParsedFile};
use crate::error::AppResult;
use crate::media::{csv_format, mal_xml, transfer_file};
use crate::state::AppState;
use tauri::State;

fn parse_file(path: &str, format: ImportFormat) -> AppResult<ParsedFile> {
    let bytes = transfer_file::read_limited(path)?;
    match format {
        ImportFormat::Csv => csv_format::parse(&bytes),
        ImportFormat::Mal => mal_xml::parse(&bytes),
    }
}

/// Reads and checks the file and marks duplicates; nothing is written.
#[tauri::command]
pub async fn media_import_preview(
    state: State<'_, AppState>,
    path: String,
    format: ImportFormat,
) -> AppResult<ImportPreview> {
    media_import::preview(&state.db, parse_file(&path, format)?).await
}

/// Reads the file again and adds the new rows, so the file may not change between steps unnoticed.
#[tauri::command]
pub async fn media_import_confirm(
    state: State<'_, AppState>,
    path: String,
    format: ImportFormat,
) -> AppResult<ImportSummary> {
    media_import::import(&state.db, parse_file(&path, format)?).await
}

#[tauri::command]
pub fn media_export_exists(folder: String) -> AppResult<bool> {
    transfer_file::export_exists(&folder)
}

/// Writes the library as CSV into the folder; returns the file path.
#[tauri::command]
pub async fn media_export_csv(
    state: State<'_, AppState>,
    folder: String,
    overwrite: bool,
) -> AppResult<String> {
    let entries = media::list(&state.db).await?;
    transfer_file::write_export(&folder, &csv_format::export(&entries)?, overwrite)
}
