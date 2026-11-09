use crate::error::{AppError, AppResult};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum AssetKind {
    Screenshot,
    Link,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProjectAsset {
    pub id: String,
    pub created_at: i64,
    pub project_id: String,
    pub kind: AssetKind,
    pub path_or_url: String,
    pub caption: String,
    pub order_key: String,
}

/// A web address a browser can open; anything else is rejected.
pub fn clean_link(url: &str) -> AppResult<String> {
    let url = url.trim();
    let host = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or_default();
    if host.is_empty() || url.contains(char::is_whitespace) {
        return Err(AppError::Invalid(
            "a link must start with http:// or https://",
        ));
    }
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_web_addresses() {
        assert_eq!(
            clean_link("  https://example.com/a?b=1 ").expect("link"),
            "https://example.com/a?b=1"
        );
        assert!(clean_link("http://localhost:3000").is_ok());
    }

    #[test]
    fn rejects_everything_else() {
        for url in [
            "",
            "example.com",
            "https://",
            "ftp://example.com",
            "javascript:alert(1)",
            "file:///c:/secret",
            "https://exa mple.com",
        ] {
            assert!(clean_link(url).is_err(), "{url:?}");
        }
    }
}
