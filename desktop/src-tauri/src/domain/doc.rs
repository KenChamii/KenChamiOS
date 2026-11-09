use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum DocKind {
    Idea,
    Plan,
    Adr,
    Note,
}

impl DocKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idea => "idea",
            Self::Plan => "plan",
            Self::Adr => "adr",
            Self::Note => "note",
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProjectDoc {
    pub id: String,
    pub created_at: i64,
    pub project_id: String,
    pub kind: DocKind,
    pub title: String,
    pub body_md: String,
}
