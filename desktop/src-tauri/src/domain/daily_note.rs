use serde::Serialize;
use sqlx::FromRow;

/// The Markdown note of one local day.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DailyNote {
    pub id: String,
    pub day: String,
    pub body: String,
}

/// `addition` below `body`, separated by an empty line.
pub fn append(body: &str, addition: &str) -> String {
    let body = body.trim_end();
    if body.is_empty() {
        addition.to_string()
    } else {
        format!("{body}\n\n{addition}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_after_an_empty_line() {
        assert_eq!(append("", "Done"), "Done");
        assert_eq!(append("Morning\n\n", "Evening"), "Morning\n\nEvening");
    }
}
