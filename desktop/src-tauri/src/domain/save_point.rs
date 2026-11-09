use super::project::Stage;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SavePoint {
    pub id: String,
    pub created_at: i64,
    pub project_id: String,
    pub stage: Stage,
    pub source: String,
    pub note: String,
    pub done_md: String,
    pub next_md: String,
    pub session_summary_md: String,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SavePointInput {
    pub note: String,
    #[serde(default)]
    pub done_md: String,
    #[serde(default)]
    pub next_md: String,
}
