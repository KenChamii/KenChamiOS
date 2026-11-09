use super::safe_path::resolve_inside;
use crate::domain::portfolio::IMAGES_DIR;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

const MARKDOWN_FILE: &str = "portfolio.md";

/// A stored screenshot to copy next to the markdown file.
pub struct ScreenshotFile {
    pub stored_name: String,
    pub source: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub output_dir: String,
    pub screenshots: usize,
}

/// Writes `<folder>/<slug>/portfolio.md` and `<folder>/<slug>/images/*`.
/// Everything is checked before the first write, and nothing leaves `<folder>/<slug>`.
pub fn export(
    folder: &str,
    slug: &str,
    markdown: &str,
    screenshots: &[ScreenshotFile],
) -> AppResult<ExportResult> {
    let markdown_path = resolve_inside(folder, &format!("{slug}/{MARKDOWN_FILE}"))?;
    let mut copies = Vec::with_capacity(screenshots.len());
    for shot in screenshots {
        if !shot.source.is_file() {
            return Err(AppError::Invalid(
                "a screenshot file is missing from the app folder, remove it and add it again",
            ));
        }
        let target = resolve_inside(folder, &format!("{slug}/{IMAGES_DIR}/{}", shot.stored_name))?;
        copies.push((&shot.source, target));
    }

    let output_dir = markdown_path
        .parent()
        .ok_or(AppError::Invalid("the export folder is not valid"))?;
    fs::create_dir_all(output_dir)?;
    // the template output has no final newline
    fs::write(&markdown_path, format!("{}\n", markdown.trim_end()))?;
    if !copies.is_empty() {
        fs::create_dir_all(output_dir.join(IMAGES_DIR))?;
    }
    for (source, target) in &copies {
        fs::copy(source, target)?;
    }
    Ok(ExportResult {
        output_dir: output_dir.to_string_lossy().into_owned(),
        screenshots: copies.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "kenchamios-portfolio-{name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }

        fn folder(&self) -> &str {
            self.0.to_str().expect("utf-8 temp path")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn shot(temp: &TempDir, name: &str, bytes: &[u8]) -> ScreenshotFile {
        let source = temp.0.join(format!("source-{name}"));
        fs::write(&source, bytes).expect("write source");
        ScreenshotFile {
            stored_name: name.to_string(),
            source,
        }
    }

    #[test]
    fn writes_the_markdown_and_copies_screenshots() {
        let temp = TempDir::new("write");
        let out = temp.0.join("export");
        fs::create_dir_all(&out).expect("export folder");
        let shots = [shot(&temp, "a.png", b"one"), shot(&temp, "b.png", b"two")];

        let result =
            export(out.to_str().expect("utf-8"), "my-app", "# Hi\n", &shots).expect("export");

        assert_eq!(result.screenshots, 2);
        assert_eq!(
            fs::read_to_string(out.join("my-app/portfolio.md")).expect("markdown"),
            "# Hi\n"
        );
        assert_eq!(
            fs::read(out.join("my-app/images/a.png")).expect("a"),
            b"one"
        );
        assert_eq!(
            fs::read(out.join("my-app/images/b.png")).expect("b"),
            b"two"
        );
    }

    #[test]
    fn exporting_again_replaces_only_the_generated_files() {
        let temp = TempDir::new("again");
        let out = temp.0.join("export");
        fs::create_dir_all(out.join("my-app")).expect("folder");
        fs::write(out.join("my-app/notes.txt"), "mine").expect("seed");
        fs::write(out.join("my-app/portfolio.md"), "old").expect("seed");

        export(out.to_str().expect("utf-8"), "my-app", "new", &[]).expect("export");

        assert_eq!(
            fs::read_to_string(out.join("my-app/portfolio.md")).expect("md"),
            "new\n"
        );
        assert_eq!(
            fs::read_to_string(out.join("my-app/notes.txt")).expect("notes"),
            "mine"
        );
        assert!(!out.join("my-app/images").exists());
    }

    #[test]
    fn paths_that_climb_out_are_rejected_before_anything_is_written() {
        let temp = TempDir::new("escape");
        let out = temp.0.join("export");
        fs::create_dir_all(&out).expect("export folder");
        let folder = out.to_str().expect("utf-8");

        assert!(export(folder, "../outside", "x", &[]).is_err());
        assert!(export(folder, "", "x", &[]).is_err());

        let sneaky = ScreenshotFile {
            stored_name: "../../escape.png".to_string(),
            ..shot(&temp, "ok.png", b"x")
        };
        let result = export(folder, "my-app", "x", &[sneaky]);

        assert!(matches!(result, Err(AppError::Invalid(_))));
        assert!(!temp.0.join("outside").exists());
        assert!(!temp.0.join("escape.png").exists());
        assert!(!out.join("my-app").exists(), "nothing is written on error");
    }

    #[test]
    fn a_missing_screenshot_stops_the_export() {
        let temp = TempDir::new("missing");
        let missing = ScreenshotFile {
            stored_name: "gone.png".to_string(),
            source: temp.0.join("gone.png"),
        };

        let result = export(temp.folder(), "my-app", "x", &[missing]);

        assert!(matches!(result, Err(AppError::Invalid(_))));
        assert!(!temp.0.join("my-app").exists());
    }

    #[test]
    fn a_missing_export_folder_is_rejected() {
        let temp = TempDir::new("nofolder");
        let gone = temp.0.join("nope");

        let result = export(gone.to_str().expect("utf-8"), "my-app", "x", &[]);

        assert!(matches!(result, Err(AppError::Invalid(_))));
    }
}
