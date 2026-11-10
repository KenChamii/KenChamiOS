use super::now_ms;
use crate::error::{AppError, AppResult};
use sqlx::{AssertSqlSafe, SqlitePool};
use std::path::{Path, PathBuf};

/// Copies the live database to `dir` as `<prefix><ms>.db` (SQLite online backup)
/// and keeps only the newest `keep` copies with that prefix.
pub async fn create(
    pool: &SqlitePool,
    dir: &Path,
    prefix: &str,
    keep: usize,
) -> AppResult<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let target = dir.join(format!("{prefix}{}.db", now_ms()));
    let text = target
        .to_str()
        .ok_or(AppError::Invalid("backup path is not valid text"))?;
    if target.exists() {
        std::fs::remove_file(&target)?;
    }
    // the target is a file name we built, a quote in the folder name is doubled
    let sql = format!("VACUUM INTO '{}'", text.replace('\x27', "''"));
    sqlx::query(AssertSqlSafe(sql)).execute(pool).await?;
    prune(dir, prefix, keep)?;
    Ok(target)
}

/// The copies in `dir` with `prefix`, oldest first.
pub fn list(dir: &Path, prefix: &str) -> AppResult<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut copies: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(prefix) && n.ends_with(".db"))
        })
        .collect();
    // the names hold a millisecond timestamp, so the text order is the age order
    copies.sort();
    Ok(copies)
}

fn prune(dir: &Path, prefix: &str, keep: usize) -> AppResult<()> {
    let copies = list(dir, prefix)?;
    let excess = copies.len().saturating_sub(keep);
    for old in copies.into_iter().take(excess) {
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
        let copies = dir.join("copies");
        std::fs::create_dir_all(&copies).expect("dir");
        std::fs::write(copies.join("other-1.db"), b"not ours").expect("other file");

        let mut last = PathBuf::new();
        for _ in 0..5 {
            last = create(&pool, &copies, "test-", 3).await.expect("backup");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        assert_eq!(list(&copies, "test-").expect("list").len(), 3);
        assert!(copies.join("other-1.db").exists(), "other prefixes stay");
        assert!(last.exists());
        assert!(std::fs::metadata(&last).expect("meta").len() > 0);
        pool.close().await;
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
