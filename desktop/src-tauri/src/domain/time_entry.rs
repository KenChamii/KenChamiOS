use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum EntryKind {
    Focus,
    Break,
    Manual,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TimeEntry {
    pub id: String,
    pub created_at: i64,
    pub project_id: String,
    pub task_id: Option<String>,
    pub kind: EntryKind,
    pub started_at: i64,
    pub ended_at: i64,
    pub duration_s: i64,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntryInput {
    pub task_id: Option<String>,
    pub started_at: i64,
    pub duration_s: i64,
    #[serde(default)]
    pub note: String,
}
