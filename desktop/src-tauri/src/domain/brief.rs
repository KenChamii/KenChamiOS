use crate::error::{AppError, AppResult};
use serde::Serialize;
use similar::{ChangeTag, TextDiff};

pub const START_MARKER: &str = "<!-- kenchamios:start -->";
pub const END_MARKER: &str = "<!-- kenchamios:end -->";

/// Files a brief can be exported to, relative to the repo.
pub const TARGETS: [&str; 3] = ["CLAUDE.md", "AGENTS.md", "docs/PROJECT_BRIEF.md"];
pub const DEFAULT_TARGET: &str = TARGETS[0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Equal,
    Insert,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub text: String,
}

pub fn ensure_known_target(target: &str) -> AppResult<()> {
    if TARGETS.contains(&target) {
        Ok(())
    } else {
        Err(AppError::Invalid("unknown brief target file"))
    }
}

/// The file content after writing `rendered` into the marked section.
/// Text outside the markers is never touched; a file without markers gets the section appended.
pub fn merge(existing: Option<&str>, rendered: &str) -> AppResult<String> {
    let section = format!("{START_MARKER}\n{}\n{END_MARKER}", rendered.trim());
    let Some(existing) = existing.filter(|text| !text.is_empty()) else {
        return Ok(format!("{section}\n"));
    };
    match (existing.find(START_MARKER), existing.find(END_MARKER)) {
        (Some(start), Some(end)) if start < end => {
            let after = end + END_MARKER.len();
            Ok(format!(
                "{}{section}{}",
                &existing[..start],
                &existing[after..]
            ))
        }
        (None, None) => {
            let separator = if existing.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            Ok(format!("{existing}{separator}{section}\n"))
        }
        _ => Err(AppError::Invalid(
            "the start and end markers in the target file do not match up",
        )),
    }
}

pub fn diff(old: &str, new: &str) -> Vec<DiffLine> {
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|change| DiffLine {
            kind: match change.tag() {
                ChangeTag::Equal => DiffKind::Equal,
                ChangeTag::Insert => DiffKind::Insert,
                ChangeTag::Delete => DiffKind::Delete,
            },
            text: change.value().trim_end_matches(['\r', '\n']).to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECTION: &str = "<!-- kenchamios:start -->\nnew brief\n<!-- kenchamios:end -->";

    #[test]
    fn new_file_is_just_the_section() {
        assert_eq!(
            merge(None, "new brief\n").expect("merge"),
            format!("{SECTION}\n")
        );
        assert_eq!(
            merge(Some(""), "new brief").expect("merge"),
            format!("{SECTION}\n")
        );
    }

    #[test]
    fn file_with_markers_only_changes_the_section() {
        let existing =
            "# Mine\n\n<!-- kenchamios:start -->\nold\n<!-- kenchamios:end -->\n\nTail\n";
        let merged = merge(Some(existing), "new brief").expect("merge");
        assert_eq!(merged, format!("# Mine\n\n{SECTION}\n\nTail\n"));
        assert_eq!(merge(Some(&merged), "new brief").expect("again"), merged);
    }

    #[test]
    fn file_without_markers_gets_the_section_appended() {
        let merged = merge(Some("# Mine\n"), "new brief").expect("merge");
        assert_eq!(merged, format!("# Mine\n\n{SECTION}\n"));
        let no_newline = merge(Some("# Mine"), "new brief").expect("merge");
        assert_eq!(no_newline, format!("# Mine\n\n{SECTION}\n"));
    }

    #[test]
    fn unbalanced_markers_are_rejected() {
        let cases = [
            "text\n<!-- kenchamios:start -->\nno end",
            "<!-- kenchamios:end -->\nthen\n<!-- kenchamios:start -->",
            "only an end\n<!-- kenchamios:end -->",
        ];
        for existing in cases {
            assert!(matches!(
                merge(Some(existing), "x"),
                Err(AppError::Invalid(_))
            ));
        }
    }

    #[test]
    fn diff_marks_inserted_and_deleted_lines() {
        let lines = diff("a\nb\n", "a\nc\n");
        let kinds: Vec<(DiffKind, &str)> =
            lines.iter().map(|l| (l.kind, l.text.as_str())).collect();
        assert_eq!(
            kinds,
            [
                (DiffKind::Equal, "a"),
                (DiffKind::Delete, "b"),
                (DiffKind::Insert, "c")
            ]
        );
    }

    #[test]
    fn only_known_targets_are_accepted() {
        assert!(ensure_known_target("AGENTS.md").is_ok());
        assert!(ensure_known_target("docs/PROJECT_BRIEF.md").is_ok());
        assert!(ensure_known_target(".git/hooks/pre-commit").is_err());
        assert!(ensure_known_target("../CLAUDE.md").is_err());
    }
}
