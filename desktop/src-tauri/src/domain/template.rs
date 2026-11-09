use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum TemplateKind {
    Brief,
    TaskPrompt,
    Portfolio,
}

impl TemplateKind {
    /// The shipped body, or None for kinds that have no default yet.
    pub fn default_body(self) -> Option<&'static str> {
        match self {
            Self::Brief => Some(include_str!("../../templates/brief.md.j2")),
            Self::TaskPrompt => Some(include_str!("../../templates/task_prompt.md.j2")),
            Self::Portfolio => None,
        }
    }

    /// Kinds that are seeded on first run.
    pub const SEEDED: [Self; 2] = [Self::Brief, Self::TaskPrompt];
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ExportTemplate {
    pub id: String,
    pub created_at: i64,
    pub kind: TemplateKind,
    pub name: String,
    pub body: String,
    pub is_default: bool,
}
