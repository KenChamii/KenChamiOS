use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Prompt {
    pub id: String,
    pub created_at: i64,
    pub project_id: Option<String>,
    pub title: String,
    pub body_md: String,
    pub tags: Json<Vec<String>>,
    pub favorite: bool,
    pub use_count: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptInput {
    pub project_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub body_md: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
