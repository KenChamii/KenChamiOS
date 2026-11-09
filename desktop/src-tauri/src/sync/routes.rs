use super::apply::apply_changes;
use super::codes::{device_token, secrets_equal, sha256_hex};
use super::export::{latest_seq, read_page};
use super::limiter::FailureLimiter;
use super::pairing::{Attempt, PairingWindow};
use super::purge::purge_tombstones;
use super::schema::SyncSchema;
use super::status::{Notifier, SyncStatus};
use super::types::{
    MAX_CHANGES, PairRequest, PairResponse, PullResponse, PushRequest, PushResponse, SyncError,
};
use super::validate::is_valid_id;
use crate::db::paired_devices::{self, PairedDevice};
use crate::db::{Db, now_ms};
use axum::extract::{ConnectInfo, DefaultBodyLimit, FromRequestParts, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tower::limit::ConcurrencyLimitLayer;

const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;
const MAX_CONNECTIONS: usize = 16;
const NAME_MAX_LEN: usize = 64;

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<Db>,
    pub schema: Arc<SyncSchema>,
    pub pairing: Arc<PairingWindow>,
    pub limiter: Arc<FailureLimiter>,
    pub status: Arc<SyncStatus>,
    pub notifier: Arc<Notifier>,
    pub backups_dir: Arc<PathBuf>,
}

pub fn router(state: ServerState) -> Router {
    Router::new()
        .route("/pair", post(pair))
        .route("/sync/push", post(push))
        .route("/sync/pull", get(pull))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(ConcurrencyLimitLayer::new(MAX_CONNECTIONS))
        .with_state(state)
}

pub struct ApiError(StatusCode, String);

impl ApiError {
    fn new(status: StatusCode, message: &str) -> Self {
        Self(status, message.to_string())
    }

    fn storage() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "storage error")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

fn peer_ip(parts: &Parts) -> Result<std::net::IpAddr, ApiError> {
    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip())
        .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "unknown peer"))
}

/// A request from a paired device; anything else is refused before the body is read.
pub struct Authed(pub PairedDevice);

impl FromRequestParts<ServerState> for Authed {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let ip = peer_ip(parts)?;
        if state.limiter.is_blocked(ip, Instant::now()) {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "too many failed attempts",
            ));
        }
        match authenticate(&state.db, &parts.headers).await {
            Some(device) => Ok(Self(device)),
            None => {
                state.limiter.record_failure(ip, Instant::now());
                Err(ApiError::new(StatusCode::UNAUTHORIZED, "not paired"))
            }
        }
    }
}

// unknown, malformed and revoked devices all look the same to the caller
async fn authenticate(db: &Db, headers: &HeaderMap) -> Option<PairedDevice> {
    let value = headers.get("authorization")?.to_str().ok()?;
    let (device_id, secret) = value.strip_prefix("Bearer ")?.split_once('.')?;
    if !is_valid_id(device_id) {
        return None;
    }
    let device = paired_devices::get(db, device_id).await.ok()?;
    let supplied = sha256_hex(secret.as_bytes());
    let stored = device.as_ref().map_or("", |d| d.token_hash.as_str());
    let matches = secrets_equal(&supplied, stored);
    device.filter(|_| matches)
}

async fn pair(
    State(state): State<ServerState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<PairRequest>,
) -> Result<Json<PairResponse>, ApiError> {
    let ip = peer.ip();
    if state.limiter.is_blocked(ip, Instant::now()) {
        return Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too many failed attempts",
        ));
    }
    match state.pairing.attempt(&request.code, Instant::now()) {
        Attempt::Closed => return Err(ApiError::new(StatusCode::NOT_FOUND, "pairing is closed")),
        Attempt::Rejected => {
            state.limiter.record_failure(ip, Instant::now());
            return Err(ApiError::new(StatusCode::FORBIDDEN, "wrong code"));
        }
        Attempt::Accepted => {}
    }
    let name = request.device_name.trim();
    if !is_valid_id(&request.device_id) || name.is_empty() || name.len() > NAME_MAX_LEN {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "invalid device"));
    }
    let token = device_token().map_err(|_| ApiError::storage())?;
    paired_devices::add(
        &state.db,
        &request.device_id,
        name,
        &sha256_hex(token.as_bytes()),
    )
    .await
    .map_err(|_| ApiError::storage())?;
    state.notifier.notify();
    Ok(Json(PairResponse {
        token,
        server_device_id: state.db.device_id.clone(),
    }))
}

async fn push(
    State(state): State<ServerState>,
    Authed(device): Authed,
    Json(request): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiError> {
    let outcome = apply_changes(
        &state.db,
        &state.schema,
        &request.changes,
        Some(state.backups_dir.as_path()),
    )
    .await
    .map_err(|error| {
        let (status, message) = describe(&error);
        state.status.record_error(&message);
        ApiError(status, message)
    })?;
    paired_devices::record_sync(&state.db, &device.device_id, None)
        .await
        .map_err(|_| ApiError::storage())?;
    state.status.clear();
    if outcome.applied > 0 {
        state.notifier.notify();
    }
    Ok(Json(PushResponse {
        applied: outcome.applied,
        ignored: outcome.ignored,
    }))
}

fn describe(error: &SyncError) -> (StatusCode, String) {
    match error {
        SyncError::Rejected(reason) => (StatusCode::UNPROCESSABLE_ENTITY, reason.clone()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage error".to_string(),
        ),
    }
}

fn storage_failure(status: &SyncStatus) -> ApiError {
    status.record_error("could not read changes");
    ApiError::storage()
}

#[derive(Deserialize)]
struct PullParams {
    since: Option<i64>,
    limit: Option<usize>,
}

async fn pull(
    State(state): State<ServerState>,
    Authed(device): Authed,
    Query(params): Query<PullParams>,
) -> Result<Json<PullResponse>, ApiError> {
    let since = params.since.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(MAX_CHANGES).clamp(1, MAX_CHANGES);
    let latest = latest_seq(&state.db)
        .await
        .map_err(|_| storage_failure(&state.status))?;
    // the cursor a device sends is also its receipt for everything before it
    paired_devices::record_sync(&state.db, &device.device_id, Some(since.min(latest)))
        .await
        .map_err(|_| storage_failure(&state.status))?;
    let page = read_page(&state.db, &state.schema, since, limit, &device.device_id)
        .await
        .map_err(|_| storage_failure(&state.status))?;
    if purge_tombstones(&state.db, now_ms()).await.is_err() {
        state.status.record_error("could not purge old deletions");
    }
    Ok(Json(PullResponse {
        changes: page.changes,
        next: page.next,
        has_more: page.has_more,
    }))
}
