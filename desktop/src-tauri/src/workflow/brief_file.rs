use crate::domain::brief::{self, DiffLine};
use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct BriefPreview {
    pub target: String,
    pub file_exists: bool,
    pub rendered: String,
    pub diff: Vec<DiffLine>,
    pub changed: bool,
}

pub fn preview(repo_path: &str, target: &str, rendered: &str) -> AppResult<BriefPreview> {
    let path = resolve_target(repo_path, target)?;
    let existing = read_existing(&path)?;
    let proposed = brief::merge(existing.as_deref(), rendered)?;
    let old = existing.as_deref().unwrap_or_default();
    Ok(BriefPreview {
        target: target.to_string(),
        file_exists: existing.is_some(),
        rendered: rendered.to_string(),
        diff: brief::diff(old, &proposed),
        changed: old != proposed,
    })
}

/// Writes the brief into the marked section of the target file inside the repo.
pub fn write(repo_path: &str, target: &str, rendered: &str) -> AppResult<()> {
    let path = resolve_target(repo_path, target)?;
    let merged = brief::merge(read_existing(&path)?.as_deref(), rendered)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, merged)?;
    Ok(())
}

/// The absolute path of `target`, which must be a plain relative path that stays inside the repo.
fn resolve_target(repo_path: &str, target: &str) -> AppResult<PathBuf> {
    let relative = Path::new(target);
    let plain = relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)));
    if target.trim().is_empty() || !plain {
        return Err(AppError::Invalid(
            "the target file must be a relative path inside the repository",
        ));
    }
    let root = fs::canonicalize(repo_path)
        .map_err(|_| AppError::Invalid("the repository folder does not exist"))?;
    if !root.is_dir() {
        return Err(AppError::Invalid("the repository path is not a folder"));
    }
    let path = root.join(relative);
    // a symlink anywhere on the way must not lead out of the repo
    let existing = path.ancestors().find(|p| p.exists()).unwrap_or(&root);
    if !fs::canonicalize(existing)?.starts_with(&root) {
        return Err(AppError::Invalid(
            "the target file resolves outside the repository",
        ));
    }
    Ok(path)
}

fn read_existing(path: &Path) -> AppResult<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::brief::DiffKind;

    struct TempRepo(PathBuf);

    impl TempRepo {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("kenchamios-brief-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp repo");
            Self(dir)
        }

        fn path(&self) -> &str {
            self.0.to_str().expect("utf-8 temp path")
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const SECTION: &str = "<!-- kenchamios:start -->\nbrief\n<!-- kenchamios:end -->\n";

    #[test]
    fn creates_a_new_file_with_parent_folders() {
        let repo = TempRepo::new("new");

        let preview = preview(repo.path(), "docs/PROJECT_BRIEF.md", "brief").expect("preview");
        assert!(!preview.file_exists);
        assert!(preview.diff.iter().all(|l| l.kind == DiffKind::Insert));
        assert!(!repo.0.join("docs").exists(), "preview must not write");

        write(repo.path(), "docs/PROJECT_BRIEF.md", "brief").expect("write");
        let written = fs::read_to_string(repo.0.join("docs/PROJECT_BRIEF.md")).expect("read");
        assert_eq!(written, SECTION);
    }

    #[test]
    fn updates_only_the_marked_section_of_an_existing_file() {
        let repo = TempRepo::new("markers");
        let file = repo.0.join("CLAUDE.md");
        fs::write(
            &file,
            "# Mine\n<!-- kenchamios:start -->\nold\n<!-- kenchamios:end -->\nTail\n",
        )
        .expect("seed");

        let preview = preview(repo.path(), "CLAUDE.md", "brief").expect("preview");
        assert!(preview.file_exists && preview.changed);
        assert_eq!(
            fs::read_to_string(&file).expect("read"),
            "# Mine\n<!-- kenchamios:start -->\nold\n<!-- kenchamios:end -->\nTail\n",
            "preview must not write"
        );

        write(repo.path(), "CLAUDE.md", "brief").expect("write");
        assert_eq!(
            fs::read_to_string(&file).expect("read"),
            format!("# Mine\n{SECTION}Tail\n")
        );
    }

    #[test]
    fn appends_a_section_to_a_file_without_markers() {
        let repo = TempRepo::new("append");
        let file = repo.0.join("AGENTS.md");
        fs::write(&file, "Hand written.\n").expect("seed");

        write(repo.path(), "AGENTS.md", "brief").expect("write");

        assert_eq!(
            fs::read_to_string(&file).expect("read"),
            format!("Hand written.\n\n{SECTION}")
        );
    }

    #[test]
    fn unchanged_brief_reports_no_changes() {
        let repo = TempRepo::new("same");
        write(repo.path(), "CLAUDE.md", "brief").expect("write");

        let preview = preview(repo.path(), "CLAUDE.md", "brief").expect("preview");

        assert!(!preview.changed);
    }

    #[test]
    fn targets_outside_the_repo_are_rejected() {
        let repo = TempRepo::new("escape");
        let outside =
            std::env::temp_dir().join(format!("kenchamios-outside-{}.md", std::process::id()));
        let absolute = outside.to_str().expect("utf-8 temp path").to_string();
        let cases = [
            "../escape.md",
            "docs/../../escape.md",
            "",
            "  ",
            "./CLAUDE.md",
            &absolute,
        ];

        for target in cases {
            let result = write(repo.path(), target, "brief");
            assert!(matches!(result, Err(AppError::Invalid(_))), "{target:?}");
            assert!(preview(repo.path(), target, "brief").is_err(), "{target:?}");
        }
        assert!(!outside.exists());
        assert!(!repo.0.parent().expect("parent").join("escape.md").exists());
    }

    #[test]
    fn missing_repo_folder_is_rejected() {
        let repo = TempRepo::new("missing");
        let gone = repo.0.join("nope");
        let result = write(gone.to_str().expect("utf-8"), "CLAUDE.md", "brief");
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[test]
    fn broken_markers_leave_the_file_alone() {
        let repo = TempRepo::new("broken");
        let file = repo.0.join("CLAUDE.md");
        fs::write(&file, "<!-- kenchamios:start -->\nno end\n").expect("seed");

        assert!(write(repo.path(), "CLAUDE.md", "brief").is_err());
        assert_eq!(
            fs::read_to_string(&file).expect("read"),
            "<!-- kenchamios:start -->\nno end\n"
        );
    }
}
