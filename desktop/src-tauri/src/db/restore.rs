use crate::error::{AppError, AppResult};
use std::io::Read;
use std::path::{Path, PathBuf};

// the open database cannot be replaced, so a restore is staged and applied on the next start
const PENDING_FILE: &str = "restore-pending";
const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";

/// Remembers `backup` as the database to use from the next start on.
pub fn stage(data_dir: &Path, backup: &Path) -> AppResult<()> {
    let mut header = [0u8; 16];
    std::fs::File::open(backup)?.read_exact(&mut header)?;
    if &header != SQLITE_HEADER {
        return Err(AppError::Invalid("that file is not a database backup"));
    }
    let text = backup
        .to_str()
        .ok_or(AppError::Invalid("backup path is not valid text"))?;
    std::fs::write(data_dir.join(PENDING_FILE), text)?;
    Ok(())
}

/// Swaps in a staged backup before the database is opened. The replaced database
/// is kept next to it as `<name>.before-restore`.
pub fn apply_pending(data_dir: &Path, db_path: &Path) -> AppResult<bool> {
    let pending = data_dir.join(PENDING_FILE);
    if !pending.exists() {
        return Ok(false);
    }
    let backup = PathBuf::from(std::fs::read_to_string(&pending)?);
    // removed first, so a broken backup cannot make every start fail
    std::fs::remove_file(&pending)?;
    if db_path.exists() {
        std::fs::copy(db_path, with_suffix(db_path, ".before-restore"))?;
    }
    let incoming = with_suffix(db_path, ".restoring");
    std::fs::copy(&backup, &incoming)?;
    std::fs::rename(&incoming, db_path)?;
    for stale in ["-wal", "-shm", "-journal"] {
        let path = with_suffix(db_path, stale);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
    }
    Ok(true)
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, snapshot};

    #[tokio::test]
    async fn a_staged_backup_replaces_the_database_on_the_next_start() {
        let dir = std::env::temp_dir().join(format!("kenchamios-restore-{}", db::new_id()));
        let db_path = dir.join("live.db");
        let pool = db::open(&db_path).await.expect("open");
        sqlx::query("INSERT INTO settings (id, created_at, updated_at, device_id, key, value_json) VALUES ('s', 1, 1, 'd', 'marker', '\"old\"')")
            .execute(&pool)
            .await
            .expect("seed");
        let backup = snapshot::create(&pool, &dir.join("copies"), "daily-", 3)
            .await
            .expect("backup");
        sqlx::query("UPDATE settings SET value_json = '\"new\"' WHERE key = 'marker'")
            .execute(&pool)
            .await
            .expect("change");
        pool.close().await;

        assert!(!apply_pending(&dir, &db_path).expect("nothing staged"));
        stage(&dir, &backup).expect("stage");
        assert!(apply_pending(&dir, &db_path).expect("apply"));
        assert!(!apply_pending(&dir, &db_path).expect("applied once"));

        let pool = db::open(&db_path).await.expect("reopen");
        let marker: String =
            sqlx::query_scalar("SELECT value_json FROM settings WHERE key = 'marker'")
                .fetch_one(&pool)
                .await
                .expect("marker");
        assert_eq!(marker, "\"old\"");
        assert!(with_suffix(&db_path, ".before-restore").exists());
        pool.close().await;
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn only_sqlite_files_can_be_staged() {
        let dir = std::env::temp_dir().join(format!("kenchamios-restore-{}", db::new_id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let fake = dir.join("fake.db");
        std::fs::write(&fake, b"definitely not a database").expect("write");
        assert!(matches!(stage(&dir, &fake), Err(AppError::Invalid(_))));
        assert!(!dir.join(PENDING_FILE).exists());
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
