use crate::error::{AppError, AppResult};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// The absolute path of `relative`, which must be a plain relative path that stays inside `root`.
pub fn resolve_inside(root: &str, relative: &str) -> AppResult<PathBuf> {
    let relative_path = Path::new(relative);
    let plain = relative_path
        .components()
        .all(|part| matches!(part, Component::Normal(_)));
    if relative.trim().is_empty() || !plain {
        return Err(AppError::Invalid(
            "the file must be a relative path inside the target folder",
        ));
    }
    let root = fs::canonicalize(root)
        .map_err(|_| AppError::Invalid("the target folder does not exist"))?;
    if !root.is_dir() {
        return Err(AppError::Invalid("the target path is not a folder"));
    }
    let path = root.join(relative_path);
    // a symlink anywhere on the way must not lead out of the folder
    let existing = path.ancestors().find(|p| p.exists()).unwrap_or(&root);
    if !fs::canonicalize(existing)?.starts_with(&root) {
        return Err(AppError::Invalid(
            "the file resolves outside the target folder",
        ));
    }
    Ok(path)
}
