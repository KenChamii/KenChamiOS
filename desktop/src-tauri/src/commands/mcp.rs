use crate::agent::brief::{self, BriefKind};
use crate::db::mcp_calls::{self, CallView};
use crate::db::{now_ms, settings};
use crate::error::{AppError, AppResult};
use crate::mcp::protocol::undo_call;
use crate::media::secrets::{KeyringStore, SecretStore};
use crate::state::AppState;
use crate::sync::codes::device_token;
use serde::Serialize;
use tauri::State;

const ENABLED_KEY: &str = "mcp.enabled";
const PORT_KEY: &str = "mcp.port";
const TOKEN_NAME: &str = "mcp_token";
pub const DEFAULT_PORT: u16 = 47_821;
const RECENT_CALLS: i64 = 30;

#[derive(Serialize)]
pub struct McpStatus {
    enabled: bool,
    running: bool,
    port: u16,
    /// shown so it can be copied into the Claude Code config
    token: Option<String>,
    calls: Vec<CallView>,
}

async fn port(state: &AppState) -> AppResult<u16> {
    Ok(settings::get_string(&state.db, PORT_KEY)
        .await?
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_PORT))
}

async fn enabled(state: &AppState) -> AppResult<bool> {
    Ok(settings::get_string(&state.db, ENABLED_KEY)
        .await?
        .as_deref()
        == Some("1"))
}

// the token lives in the credential store, never in the database
fn token() -> AppResult<String> {
    if let Some(existing) = KeyringStore.get(TOKEN_NAME)? {
        return Ok(existing);
    }
    let fresh = device_token()?;
    KeyringStore.set(TOKEN_NAME, &fresh)?;
    Ok(fresh)
}

async fn start(state: &AppState) -> AppResult<()> {
    state
        .mcp
        .start(state.db.clone(), token()?, port(state).await?)
        .await
}

/// Brings the server back at app start when the user had turned it on.
pub async fn resume(state: &AppState) {
    if matches!(enabled(state).await, Ok(true))
        && let Err(error) = start(state).await
    {
        eprintln!("MCP server did not start: {error}");
    }
}

async fn status_of(state: &AppState) -> AppResult<McpStatus> {
    let on = enabled(state).await?;
    Ok(McpStatus {
        enabled: on,
        running: state.mcp.is_running(),
        port: match state.mcp.port() {
            Some(running) => running,
            None => port(state).await?,
        },
        token: if on { Some(token()?) } else { None },
        calls: mcp_calls::list(&state.db, RECENT_CALLS).await?,
    })
}

#[tauri::command]
pub async fn mcp_status(state: State<'_, AppState>) -> AppResult<McpStatus> {
    status_of(&state).await
}

#[tauri::command]
pub async fn mcp_set_enabled(state: State<'_, AppState>, enabled: bool) -> AppResult<McpStatus> {
    if enabled {
        start(&state).await?;
    } else {
        state.mcp.stop();
    }
    settings::set_string(&state.db, ENABLED_KEY, if enabled { "1" } else { "0" }).await?;
    status_of(&state).await
}

/// A new token ends every session that used the old one.
#[tauri::command]
pub async fn mcp_new_token(state: State<'_, AppState>) -> AppResult<McpStatus> {
    KeyringStore.set(TOKEN_NAME, &device_token()?)?;
    if enabled(&state).await? {
        start(&state).await?;
    }
    status_of(&state).await
}

#[tauri::command]
pub async fn mcp_undo(state: State<'_, AppState>, id: i64) -> AppResult<String> {
    let call = mcp_calls::get(&state.db, id).await?;
    if call.undo_json.is_none() || call.undone {
        return Err(AppError::Invalid("this change cannot be undone"));
    }
    undo_call(&state.db, &call, now_ms()).await
}

#[derive(Serialize)]
pub struct BriefSetting {
    kind: BriefKind,
    enabled: bool,
}

#[tauri::command]
pub async fn briefs_status(state: State<'_, AppState>) -> AppResult<Vec<BriefSetting>> {
    let mut all = Vec::new();
    for kind in BriefKind::ALL {
        all.push(BriefSetting {
            kind,
            enabled: brief::is_enabled(&state.db, kind).await?,
        });
    }
    Ok(all)
}

#[tauri::command]
pub async fn briefs_set_enabled(
    state: State<'_, AppState>,
    kind: BriefKind,
    enabled: bool,
) -> AppResult<()> {
    brief::set_enabled(&state.db, kind, enabled).await
}

/// Writes the brief now; returns the path of the note inside the vault.
#[tauri::command]
pub async fn briefs_run(state: State<'_, AppState>, kind: BriefKind) -> AppResult<String> {
    brief::run(&state.db, &state.data_dir, kind, now_ms()).await
}
