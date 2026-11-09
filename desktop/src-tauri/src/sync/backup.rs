use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use sqlx::{AssertSqlSafe, SqlitePool};
use std::path::{Path, PathBuf};

const PREFIX: &str = "pre-sync-";
const KEEP: usize = 5;

/// Copies the live database to `dir` (SQLite online backup) and keeps the newest few copies.
pub async fn create(pool: &SqlitePool, dir: &Path) -> AppResult<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let target = dir.join(format!("{PREFIX}{}.db", now_ms()));
    let text = target
        .to_str()
        .ok_or(AppError::Invalid("backup path is not valid text"))?;
    if target.exists() {
        std::fs::remove_file(&target)?;
    }
    // the target is a file name we built, a quote in the folder name is doubled
    let sql = format!("VACUUM INTO '{}'", text.replace('\x27', "''"));
    sqlx::query(AssertSqlSafe(sql)).execute(pool).await?;
    prune(dir)?;
    Ok(target)
}

fn prune(dir: &Path) -> AppResult<()> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(PREFIX))
        })
        .collect();
    // the names hold a millisecond timestamp, so the text order is the age order
    backups.sort();
    let excess = backups.len().saturating_sub(KEEP);
    for old in backups.into_iter().take(excess) {
        std::fs::remove_file(old)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn keeps_only_the_newest_copies_and_they_hold_the_data() {
        let dir = std::env::temp_dir().join(format!("kenchamios-backup-{}", crate::db::new_id()));
        let pool = crate::db::open(&dir.join("live.db"))
            .await
            .expect("open db");

        let mut last = PathBuf::new();
        for _ in 0..(KEEP + 2) {
            last = create(&pool, &dir.join("copies")).await.expect("backup");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        assert_eq!(
            std::fs::read_dir(dir.join("copies")).expect("dir").count(),
            KEEP
        );
        assert!(last.exists());
        assert!(std::fs::metadata(&last).expect("meta").len() > 0);
        pool.close().await;
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
