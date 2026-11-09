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
    pub const ALL: [Self; 3] = [Self::Brief, Self::TaskPrompt, Self::Portfolio];

    /// The shipped body.
    pub fn default_body(self) -> &'static str {
        match self {
            Self::Brief => include_str!("../../templates/brief.md.j2"),
            Self::TaskPrompt => include_str!("../../templates/task_prompt.md.j2"),
            Self::Portfolio => include_str!("../../templates/portfolio.md.j2"),
        }
    }
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
