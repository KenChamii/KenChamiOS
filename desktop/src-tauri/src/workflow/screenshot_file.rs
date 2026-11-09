use super::safe_path::resolve_inside;
use crate::db::new_id;
use crate::error::{AppError, AppResult};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const MAX_BYTES: u64 = 25 * 1024 * 1024;
const EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

/// A screenshot copied into the app's assets folder.
#[derive(Debug, PartialEq, Eq)]
pub struct Imported {
    /// file name inside the project's folder
    pub stored_name: String,
    /// original file name without extension, a starting point for the caption
    pub stem: String,
}

/// Copies an image into `<assets_root>/<project_id>/` under a fresh name.
/// The source must be a real image file; nothing existing is ever replaced.
pub fn import(assets_root: &Path, project_id: &str, source: &str) -> AppResult<Imported> {
    let source = fs::canonicalize(source)
        .map_err(|_| AppError::Invalid("the screenshot file does not exist"))?;
    let metadata = fs::metadata(&source)?;
    if !metadata.is_file() {
        return Err(AppError::Invalid("the screenshot is not a file"));
    }
    if metadata.len() > MAX_BYTES {
        return Err(AppError::Invalid("the screenshot is larger than 25 MB"));
    }
    let extension = image_extension(&source)?;

    let mut input = File::open(&source)?;
    let mut header = [0u8; 12];
    let read = read_up_to(&mut input, &mut header)?;
    if !matches_type(&extension, &header[..read]) {
        return Err(AppError::Invalid(
            "the file content does not match an image of that type",
        ));
    }

    fs::create_dir_all(assets_root)?;
    let stored_name = format!("{}.{extension}", new_id());
    let destination = path_of(assets_root, project_id, &stored_name)?;
    if let Some(folder) = destination.parent() {
        fs::create_dir_all(folder)?;
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)?;
    let copied = io::copy(&mut io::Read::chain(&header[..read], input), &mut output);
    if let Err(error) = copied {
        // do not leave a half written file behind
        let _ = fs::remove_file(&destination);
        return Err(error.into());
    }

    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Imported { stored_name, stem })
}

/// Where a stored screenshot lives; the name must be a plain file name.
pub fn path_of(assets_root: &Path, project_id: &str, stored_name: &str) -> AppResult<PathBuf> {
    if stored_name.is_empty() || stored_name.contains(['/', '\\']) {
        return Err(AppError::Invalid("the screenshot name is not valid"));
    }
    let root = assets_root
        .to_str()
        .ok_or(AppError::Invalid("the app data folder path is not valid"))?;
    resolve_inside(root, &format!("{project_id}/{stored_name}"))
}

fn image_extension(path: &Path) -> AppResult<String> {
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if EXTENSIONS.contains(&extension.as_str()) {
        Ok(extension)
    } else {
        Err(AppError::Invalid(
            "screenshots must be png, jpg, gif or webp files",
        ))
    }
}

fn matches_type(extension: &str, header: &[u8]) -> bool {
    match extension {
        "png" => header.starts_with(&[0x89, b'P', b'N', b'G']),
        "jpg" | "jpeg" => header.starts_with(&[0xFF, 0xD8, 0xFF]),
        "gif" => header.starts_with(b"GIF8"),
        "webp" => header.starts_with(b"RIFF") && header.get(8..12) == Some(b"WEBP".as_slice()),
        _ => false,
    }
}

fn read_up_to(input: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match input.read(&mut buffer[filled..])? {
            0 => break,
            n => filled += n,
        }
    }
    Ok(filled)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("kenchamios-shots-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }

        fn write(&self, name: &str, bytes: &[u8]) -> String {
            let path = self.0.join(name);
            fs::write(&path, bytes).expect("write file");
            path.to_str().expect("utf-8 temp path").to_string()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const PNG: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3, 4, 5,
    ];

    #[test]
    fn copies_an_image_under_a_new_name() {
        let temp = TempDir::new("copy");
        let source = temp.write("Home Screen.PNG", PNG);
        let assets = temp.0.join("assets");

        let imported = import(&assets, "proj", &source).expect("import");

        assert_eq!(imported.stem, "Home Screen");
        assert!(imported.stored_name.ends_with(".png"));
        let stored = path_of(&assets, "proj", &imported.stored_name).expect("path");
        assert_eq!(fs::read(stored).expect("read copy"), PNG);
        assert_eq!(fs::read(&source).expect("source kept"), PNG);
    }

    #[test]
    fn importing_twice_never_replaces_a_copy() {
        let temp = TempDir::new("twice");
        let source = temp.write("a.png", PNG);
        let assets = temp.0.join("assets");

        let first = import(&assets, "proj", &source).expect("first");
        let second = import(&assets, "proj", &source).expect("second");

        assert_ne!(first.stored_name, second.stored_name);
        assert_eq!(fs::read_dir(assets.join("proj")).expect("dir").count(), 2);
    }

    #[test]
    fn rejects_files_that_are_not_images() {
        let temp = TempDir::new("reject");
        let assets = temp.0.join("assets");
        let cases = [
            ("notes.txt", PNG),
            ("fake.png", b"just text, not a picture".as_slice()),
            ("tiny.jpg", b"".as_slice()),
            ("noext", PNG),
        ];

        for (name, bytes) in cases {
            let source = temp.write(name, bytes);
            let result = import(&assets, "proj", &source);
            assert!(matches!(result, Err(AppError::Invalid(_))), "{name}");
        }
        assert!(
            !assets.join("proj").exists()
                || fs::read_dir(assets.join("proj")).expect("dir").count() == 0
        );
    }

    #[test]
    fn rejects_folders_and_missing_files() {
        let temp = TempDir::new("missing");
        let assets = temp.0.join("assets");
        let folder = temp.0.join("folder.png");
        fs::create_dir_all(&folder).expect("folder");

        for source in [
            folder.to_str().expect("utf-8"),
            temp.0.join("gone.png").to_str().expect("utf-8"),
        ] {
            let result = import(&assets, "proj", source);
            assert!(matches!(result, Err(AppError::Invalid(_))), "{source}");
        }
    }

    #[test]
    fn project_ids_and_names_cannot_climb_out() {
        let temp = TempDir::new("escape");
        let source = temp.write("a.png", PNG);
        let assets = temp.0.join("assets");

        assert!(import(&assets, "../outside", &source).is_err());
        assert!(path_of(&assets, "proj", "../../secret.png").is_err());
        assert!(path_of(&assets, "proj", "").is_err());
        assert!(!temp.0.join("outside").exists());
    }

    #[test]
    fn recognises_each_image_type() {
        assert!(matches_type("jpeg", &[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(matches_type("gif", b"GIF89a"));
        assert!(matches_type("webp", b"RIFF\x00\x00\x00\x00WEBP"));
        assert!(!matches_type("webp", b"RIFF\x00\x00\x00\x00WAVE"));
        assert!(!matches_type("png", b"GIF89a"));
    }
}
