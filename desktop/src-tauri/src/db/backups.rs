use super::handle::{Db, now_ms};
use super::{settings, snapshot};
use crate::domain::backup::{
    BackupFile, BackupStatus, DEFAULT_KEEP, created_at, is_due, validate_keep,
};
use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

pub const PREFIX: &str = "daily-";
const FOLDER_KEY: &str = "backup.folder";
const KEEP_KEY: &str = "backup.keep";
const LAST_AT_KEY: &str = "backup.last_at";
const LAST_ATTEMPT_KEY: &str = "backup.last_attempt_at";
const LAST_ERROR_KEY: &str = "backup.last_error";

async fn number(db: &Db, key: &str) -> AppResult<Option<i64>> {
    Ok(settings::get_string(db, key)
        .await?
        .and_then(|text| text.parse().ok()))
}

async fn custom_folder(db: &Db) -> AppResult<Option<PathBuf>> {
    Ok(settings::get_string(db, FOLDER_KEY)
        .await?
        .filter(|text| !text.is_empty())
        .map(PathBuf::from))
}

async fn keep(db: &Db) -> AppResult<i64> {
    Ok(number(db, KEEP_KEY)
        .await?
        .filter(|keep| validate_keep(*keep).is_ok())
        .unwrap_or(DEFAULT_KEEP))
}

/// Where the copies go: the folder the user picked, or `default_dir`.
pub async fn folder(db: &Db, default_dir: &Path) -> AppResult<PathBuf> {
    Ok(custom_folder(db)
        .await?
        .unwrap_or_else(|| default_dir.to_path_buf()))
}

pub async fn status(db: &Db, default_dir: &Path) -> AppResult<BackupStatus> {
    let custom = custom_folder(db).await?;
    let folder = custom.clone().unwrap_or_else(|| default_dir.to_path_buf());
    let mut copies = Vec::new();
    for path in snapshot::list(&folder, PREFIX)?.into_iter().rev() {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        copies.push(BackupFile {
            created_at: created_at(&name, PREFIX),
            size_bytes: std::fs::metadata(&path)?.len(),
            path: path.to_string_lossy().into_owned(),
            name,
        });
    }
    Ok(BackupStatus {
        folder: folder.to_string_lossy().into_owned(),
        custom_folder: custom.is_some(),
        keep: keep(db).await?,
        last_at: number(db, LAST_AT_KEY).await?,
        last_error: settings::get_string(db, LAST_ERROR_KEY)
            .await?
            .filter(|text| !text.is_empty()),
        copies,
    })
}

/// Stores the folder (blank for the default one) and how many copies to keep.
pub async fn configure(db: &Db, folder: Option<&str>, keep: i64) -> AppResult<()> {
    validate_keep(keep)?;
    let folder = folder.map(str::trim).unwrap_or_default();
    if !folder.is_empty() {
        let path = Path::new(folder);
        if !path.is_absolute() {
            return Err(AppError::Invalid("the backup folder must be a full path"));
        }
        std::fs::create_dir_all(path)?;
    }
    settings::set_string(db, FOLDER_KEY, folder).await?;
    settings::set_string(db, KEEP_KEY, &keep.to_string()).await
}

/// Takes a backup now; a failure is remembered so the settings page can show it.
pub async fn run(db: &Db, default_dir: &Path) -> AppResult<PathBuf> {
    let now = now_ms();
    settings::set_string(db, LAST_ATTEMPT_KEY, &now.to_string()).await?;
    let folder = folder(db, default_dir).await?;
    let keep = usize::try_from(keep(db).await?).unwrap_or(1);
    match snapshot::create(&db.pool, &folder, PREFIX, keep).await {
        Ok(path) => {
            settings::set_string(db, LAST_AT_KEY, &now.to_string()).await?;
            settings::set_string(db, LAST_ERROR_KEY, "").await?;
            Ok(path)
        }
        Err(error) => {
            settings::set_string(db, LAST_ERROR_KEY, &error.to_string()).await?;
            Err(error)
        }
    }
}

/// The daily backup, when one is due.
pub async fn run_if_due(db: &Db, default_dir: &Path) -> AppResult<Option<PathBuf>> {
    let last_at = number(db, LAST_AT_KEY).await?;
    let last_attempt = number(db, LAST_ATTEMPT_KEY).await?;
    if !is_due(now_ms(), last_at, last_attempt) {
        return Ok(None);
    }
    run(db, default_dir).await.map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("kenchamios-daily-{}", crate::db::new_id()))
    }

    #[tokio::test]
    async fn runs_once_a_day_into_the_chosen_folder() {
        // VACUUM INTO needs a database file, the in-memory test database writes nothing
        let live = temp_dir();
        let pool = crate::db::open(&live.join("live.db")).await.expect("open");
        let db = Db::new(pool, crate::sync::hlc::Hlc::new(), "test-device".into());
        let default_dir = temp_dir();
        let chosen = temp_dir();

        configure(&db, Some(&chosen.to_string_lossy()), 2)
            .await
            .expect("configure");
        assert!(
            run_if_due(&db, &default_dir)
                .await
                .expect("first")
                .is_some()
        );
        assert!(
            run_if_due(&db, &default_dir)
                .await
                .expect("second")
                .is_none()
        );
        run(&db, &default_dir).await.expect("manual");
        run(&db, &default_dir).await.expect("manual");

        let status = status(&db, &default_dir).await.expect("status");
        assert!(status.custom_folder);
        assert_eq!(status.keep, 2);
        assert_eq!(status.copies.len(), 2);
        assert!(status.last_at.is_some() && status.last_error.is_none());
        assert!(!default_dir.exists());
        db.pool.close().await;
        std::fs::remove_dir_all(&chosen).expect("cleanup");
        std::fs::remove_dir_all(&live).expect("cleanup");
    }

    #[tokio::test]
    async fn rejects_relative_folders_and_bad_counts() {
        let db = Db::in_memory().await;
        assert!(matches!(
            configure(&db, Some("relative/dir"), 14).await,
            Err(AppError::Invalid(_))
        ));
        assert!(matches!(
            configure(&db, None, 0).await,
            Err(AppError::Invalid(_))
        ));
        configure(&db, Some("  "), 14)
            .await
            .expect("default folder");
        let default_dir = temp_dir();
        let status = status(&db, &default_dir).await.expect("status");
        assert!(!status.custom_folder);
        assert_eq!(status.folder, default_dir.to_string_lossy());
    }
}
