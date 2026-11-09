use crate::domain::media_transfer::MAX_FILE_BYTES;
use crate::error::{AppError, AppResult};
use crate::workflow::safe_path::resolve_inside;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

pub const EXPORT_FILE: &str = "kenchamios-media.csv";

/// The whole file, as long as it is a regular file within the size limit.
pub fn read_limited(path: &str) -> AppResult<Vec<u8>> {
    let path = Path::new(path);
    let too_big = || AppError::Invalid("the file is too large (the limit is 5 MB)");
    let meta = fs::metadata(path).map_err(|_| AppError::Invalid("the file cannot be opened"))?;
    if !meta.is_file() {
        return Err(AppError::Invalid("the path is not a file"));
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err(too_big());
    }
    // the size can change after the check, so the read is capped too
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(too_big());
    }
    Ok(bytes)
}

pub fn export_exists(folder: &str) -> AppResult<bool> {
    Ok(resolve_inside(folder, EXPORT_FILE)?.exists())
}

/// Writes the export into `folder`; an existing file is only replaced when `overwrite` is set.
pub fn write_export(folder: &str, content: &str, overwrite: bool) -> AppResult<String> {
    let path = resolve_inside(folder, EXPORT_FILE)?;
    let mut options = OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(&path).map_err(|error| match error.kind() {
        ErrorKind::AlreadyExists => AppError::Invalid("the export file already exists"),
        _ => error.into(),
    })?;
    file.write_all(content.as_bytes())?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("kenchamios-transfer-{name}-{}", std::process::id()));
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

    #[test]
    fn small_files_are_read() {
        let temp = TempDir::new("read");
        let file = temp.0.join("in.csv");
        fs::write(&file, "a,b").expect("write");

        let bytes = read_limited(file.to_str().expect("path")).expect("read");

        assert_eq!(bytes, b"a,b");
    }

    #[test]
    fn large_files_folders_and_missing_files_are_refused() {
        let temp = TempDir::new("refuse");
        let big = temp.0.join("big.csv");
        fs::write(&big, vec![b'x'; MAX_FILE_BYTES as usize + 1]).expect("write");

        assert!(read_limited(big.to_str().expect("path")).is_err());
        assert!(read_limited(temp.folder()).is_err());
        assert!(read_limited(temp.0.join("none.csv").to_str().expect("path")).is_err());
    }

    #[test]
    fn exports_never_replace_a_file_without_consent() {
        let temp = TempDir::new("export");
        assert!(!export_exists(temp.folder()).expect("exists"));

        write_export(temp.folder(), "first", false).expect("first write");
        assert!(export_exists(temp.folder()).expect("exists"));
        let refused = write_export(temp.folder(), "second", false);
        assert!(refused.is_err());
        assert_eq!(
            fs::read_to_string(temp.0.join(EXPORT_FILE)).expect("read"),
            "first"
        );

        write_export(temp.folder(), "third", true).expect("overwrite");
        assert_eq!(
            fs::read_to_string(temp.0.join(EXPORT_FILE)).expect("read"),
            "third"
        );
    }

    #[test]
    fn a_missing_folder_is_refused() {
        let temp = TempDir::new("missing");
        let gone = temp.0.join("nope");

        assert!(write_export(gone.to_str().expect("path"), "x", false).is_err());
    }
}
