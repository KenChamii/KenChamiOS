use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Most changes in one push or pull page.
pub const MAX_CHANGES: usize = 500;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub table: String,
    pub row: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PushRequest {
    pub changes: Vec<Change>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushResponse {
    pub applied: usize,
    pub ignored: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<Change>,
    pub next: i64,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairRequest {
    pub code: String,
    pub device_id: String,
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairResponse {
    pub token: String,
    pub server_device_id: String,
}

/// Content of the QR code.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingPayload {
    pub v: u8,
    pub host: String,
    pub port: u16,
    pub fp: String,
    pub code: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("rejected: {0}")]
    Rejected(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    App(#[from] crate::error::AppError),
}
