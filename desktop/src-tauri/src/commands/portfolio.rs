use crate::db::{projects, templates};
use crate::domain::TemplateKind;
use crate::domain::render::render;
use crate::error::AppResult;
use crate::state::AppState;
use crate::workflow::context::portfolio_context;
use crate::workflow::portfolio_file::{self, ExportResult, ScreenshotFile};
use crate::workflow::screenshot_file;
use tauri::State;

/// The portfolio page as Markdown, for the preview and the clipboard.
#[tauri::command]
pub async fn portfolio_render(state: State<'_, AppState>, project_id: String) -> AppResult<String> {
    let template = templates::get_default(&state.db, TemplateKind::Portfolio).await?;
    let ctx = portfolio_context(&state.db, &project_id).await?;
    render(&template.body, &ctx)
}

/// Writes the page and its screenshots into `<folder>/<project slug>/`.
#[tauri::command]
pub async fn portfolio_export(
    state: State<'_, AppState>,
    project_id: String,
    folder: String,
) -> AppResult<ExportResult> {
    let template = templates::get_default(&state.db, TemplateKind::Portfolio).await?;
    let ctx = portfolio_context(&state.db, &project_id).await?;
    let markdown = render(&template.body, &ctx)?;
    let slug = projects::get(&state.db, &project_id).await?.project.slug;

    let screenshots = ctx
        .portfolio
        .iter()
        .flat_map(|portfolio| &portfolio.screenshots)
        .map(|shot| {
            Ok(ScreenshotFile {
                stored_name: shot.stored_name.clone(),
                source: screenshot_file::path_of(
                    &state.assets_dir,
                    &project_id,
                    &shot.stored_name,
                )?,
            })
        })
        .collect::<AppResult<Vec<_>>>()?;
    portfolio_file::export(&folder, &slug, &markdown, &screenshots)
}
