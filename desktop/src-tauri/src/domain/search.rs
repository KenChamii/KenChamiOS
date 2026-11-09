use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum SearchKind {
    Task,
    Idea,
    Plan,
    Prompt,
    SavePoint,
}

#[derive(Debug, Clone, FromRow)]
pub struct SearchRow {
    pub kind: SearchKind,
    pub id: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHit {
    pub kind: SearchKind,
    pub id: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub title: String,
    /// a few words around the match in the body; empty when only the title matched
    pub snippet: String,
}

const SNIPPET_CONTEXT: usize = 40;

impl SearchHit {
    pub fn from_row(row: SearchRow, query: &str) -> Self {
        Self {
            snippet: snippet(&row.body, query),
            kind: row.kind,
            id: row.id,
            project_id: row.project_id,
            project_name: row.project_name,
            title: row.title,
        }
    }
}

/// `%query%` for LIKE with a backslash escape, so `%` and `_` in the query are literal.
pub fn like_pattern(query: &str) -> String {
    let mut pattern = String::from("%");
    for c in query.chars() {
        if matches!(c, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    pattern
}

/// The text around the first match, on one line; ascii case is ignored like SQLite's LIKE.
pub fn snippet(body: &str, query: &str) -> String {
    let lower_body = body.to_ascii_lowercase();
    let Some(start) = lower_body.find(&query.to_ascii_lowercase()) else {
        return String::new();
    };
    let end = start + query.len();
    let from = floor_boundary(body, start.saturating_sub(SNIPPET_CONTEXT));
    let to = floor_boundary(body, (end + SNIPPET_CONTEXT).min(body.len()));
    let text = body[from..to]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let prefix = if from > 0 { "…" } else { "" };
    let suffix = if to < body.len() { "…" } else { "" };
    format!("{prefix}{text}{suffix}")
}

fn floor_boundary(text: &str, mut index: usize) -> usize {
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_wildcards_in_the_query_are_literal() {
        assert_eq!(like_pattern(r"50%_off\"), r"%50\%\_off\\%");
    }

    #[test]
    fn snippet_shows_the_match_with_context() {
        assert_eq!(snippet("Fix the Login bug", "login"), "Fix the Login bug");
        assert_eq!(snippet("title only", "zzz"), "");
        let long = format!("{}needle{}", "a ".repeat(40), "b ".repeat(40));
        let cut = snippet(&long, "needle");
        assert!(cut.starts_with('…') && cut.ends_with('…'));
        assert!(cut.contains("needle"));
    }

    #[test]
    fn snippet_never_splits_a_character() {
        let text = format!("{}needle{}", "é".repeat(30), "ü".repeat(30));
        assert!(snippet(&text, "needle").contains("needle"));
    }
}
