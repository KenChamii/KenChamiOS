use super::http::{HttpClient, HttpRequest};
use crate::error::{AppError, AppResult};
use crate::workflow::safe_path::resolve_inside;
use crate::workflow::screenshot_file::detect_extension;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_BYTES: usize = 5 * 1024 * 1024;

/// Downloads a cover into `covers_dir` under a fresh name and returns that file name.
/// Only https images are taken, the bytes must really be an image, and nothing is replaced.
pub async fn download<H: HttpClient>(
    http: &H,
    covers_dir: &Path,
    item_id: &str,
    url: &str,
) -> AppResult<String> {
    if !url.starts_with("https://") {
        return Err(AppError::Invalid("a cover must come from an https address"));
    }
    let request = HttpRequest {
        url: url.to_string(),
        params: Vec::new(),
        body: None,
        max_bytes: MAX_BYTES,
    };
    let bytes = http.send(request).await?.into_body()?;
    let extension = detect_extension(&bytes).ok_or(AppError::Invalid(
        "the cover is not a png, jpg, gif or webp image",
    ))?;

    fs::create_dir_all(covers_dir)?;
    let name = format!("{item_id}.{extension}");
    let path = path_of(covers_dir, &name)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    if let Err(error) = file.write_all(&bytes) {
        // do not leave a half written file behind
        let _ = fs::remove_file(&path);
        return Err(error.into());
    }
    Ok(name)
}

/// Where a cached cover lives; `None` when the name is not a plain file name in the folder.
pub fn file_of(covers_dir: &Path, name: &str) -> Option<PathBuf> {
    path_of(covers_dir, name).ok()?;
    // the plain joined path, because the checked one may carry a windows extended prefix
    Some(covers_dir.join(name))
}

fn path_of(covers_dir: &Path, name: &str) -> AppResult<PathBuf> {
    let root = covers_dir
        .to_str()
        .ok_or(AppError::Invalid("the app data folder path is not valid"))?;
    resolve_inside(root, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::test_support::FakeHttp;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("kenchamios-covers-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3];

    #[tokio::test]
    async fn stores_an_image_inside_the_covers_folder() {
        let temp = TempDir::new("store");
        let covers = temp.0.join("covers");
        let http = FakeHttp::default();
        http.push_bytes(200, PNG.to_vec());

        let name = download(&http, &covers, "item1", "https://cdn.test/a")
            .await
            .expect("download");

        assert_eq!(name, "item1.png");
        assert_eq!(fs::read(covers.join(&name)).expect("read"), PNG);
        assert_eq!(file_of(&covers, &name), Some(covers.join("item1.png")));
    }

    #[tokio::test]
    async fn rejects_non_https_and_non_image_answers() {
        let temp = TempDir::new("reject");
        let covers = temp.0.join("covers");
        let http = FakeHttp::default();
        http.push_bytes(200, b"<html>not an image</html>".to_vec());

        let plain = download(&http, &covers, "i", "http://cdn.test/a").await;
        let html = download(&http, &covers, "i", "https://cdn.test/a").await;

        assert!(matches!(plain, Err(AppError::Invalid(_))));
        assert!(matches!(html, Err(AppError::Invalid(_))));
        assert_eq!(http.requests.borrow().len(), 1);
        assert!(!covers.exists() || fs::read_dir(&covers).expect("dir").count() == 0);
    }

    #[tokio::test]
    async fn never_replaces_an_existing_file_or_escapes_the_folder() {
        let temp = TempDir::new("escape");
        let covers = temp.0.join("covers");
        let http = FakeHttp::default();
        http.push_bytes(200, PNG.to_vec());
        http.push_bytes(200, PNG.to_vec());
        download(&http, &covers, "same", "https://cdn.test/a")
            .await
            .expect("first");

        let again = download(&http, &covers, "same", "https://cdn.test/a").await;
        assert!(again.is_err());

        http.push_bytes(200, PNG.to_vec());
        let escaped = download(&http, &covers, "../outside", "https://cdn.test/a").await;
        assert!(escaped.is_err());
        assert!(!temp.0.join("outside.png").exists());
        assert!(file_of(&covers, "../x.png").is_none());
    }

    #[tokio::test]
    async fn failed_requests_leave_nothing_behind() {
        let temp = TempDir::new("fail");
        let covers = temp.0.join("covers");
        let http = FakeHttp::default();
        http.push_failure();
        http.push(404, "");

        assert!(
            download(&http, &covers, "i", "https://cdn.test/a")
                .await
                .is_err()
        );
        assert!(
            download(&http, &covers, "i", "https://cdn.test/a")
                .await
                .is_err()
        );
        assert!(!covers.exists() || fs::read_dir(&covers).expect("dir").count() == 0);
    }
}
