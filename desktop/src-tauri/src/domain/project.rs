use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;

/// Every device creates the inbox with this id, so sync merges it; see `shared/schema/well-known.json`.
pub const INBOX_ID: &str = "00000000-0000-4000-8000-000000000001";
pub const INBOX_NAME: &str = "Inbox";
pub const INBOX_SLUG: &str = "inbox";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Stage {
    Idea,
    Plan,
    Build,
    Review,
    Done,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idea => "idea",
            Self::Plan => "plan",
            Self::Build => "build",
            Self::Review => "review",
            Self::Done => "done",
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Project {
    pub id: String,
    pub created_at: i64,
    pub name: String,
    pub slug: String,
    pub stage: Stage,
    pub description: String,
    pub repo_path: Option<String>,
    pub color: Option<String>,
    pub tags: Json<Vec<String>>,
    pub archived: bool,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProjectSummary {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub project: Project,
    pub tasks_total: i64,
    pub tasks_done: i64,
    pub last_activity_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub repo_path: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct ProjectFilter {
    pub search: Option<String>,
    pub tag: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
}

/// Lowercase ascii words joined by dashes, e.g. "My App!" becomes "my-app".
pub fn slugify(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if words.is_empty() {
        "project".to_string()
    } else {
        words.join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_joins_words() {
        assert_eq!(slugify("  My App!  v2 "), "my-app-v2");
    }

    #[test]
    fn inbox_matches_the_shared_well_known_ids() {
        let path = format!(
            "{}/../../shared/schema/well-known.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(path).expect("read");
        let known: serde_json::Value = serde_json::from_str(&text).expect("parse");
        let inbox = &known["inbox_project"];
        assert_eq!(inbox["id"], INBOX_ID);
        assert_eq!(inbox["name"], INBOX_NAME);
        assert_eq!(inbox["slug"], INBOX_SLUG);
    }

    #[test]
    fn slugify_falls_back_when_empty() {
        assert_eq!(slugify("!!!"), "project");
    }
}
