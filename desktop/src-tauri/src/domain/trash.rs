use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum TrashKind {
    Project,
    Task,
    Prompt,
    SavePoint,
    Media,
    Habit,
}

/// A deleted row that can be brought back; rows deleted along with their parent are left out.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TrashItem {
    pub kind: TrashKind,
    pub id: String,
    pub title: String,
    /// the project name, or the media type
    pub detail: Option<String>,
    pub deleted_at: i64,
}
