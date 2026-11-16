use super::handle::Db;
use super::settings;
use crate::error::AppResult;

const PATH_KEY: &str = "vault.path";
const LAST_SYNC_KEY: &str = "vault.last_sync";
const ENABLED_KEY: &str = "vault.enabled";
const EXCLUDED_KEY: &str = "vault.excluded";
pub const DEFAULT_VAULT: &str = r"C:\Obsidian\KenChami Obsidian";

/// What was last written or read for one file of the vault.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct FileState {
    pub path: String,
    pub kco_id: String,
    pub hash: String,
    pub app_hash: String,
    pub mtime: i64,
    pub synced_at: i64,
}

pub async fn path(db: &Db) -> AppResult<String> {
    Ok(settings::get_string(db, PATH_KEY)
        .await?
        .unwrap_or_else(|| DEFAULT_VAULT.to_string()))
}

pub async fn set_path(db: &Db, value: &str) -> AppResult<()> {
    settings::set_string(db, PATH_KEY, value.trim()).await
}

/// Off until the user turns it on, so the app never writes into a vault unasked.
pub async fn enabled(db: &Db) -> AppResult<bool> {
    Ok(settings::get_string(db, ENABLED_KEY).await?.as_deref() == Some("1"))
}

pub async fn set_enabled(db: &Db, on: bool) -> AppResult<()> {
    settings::set_string(db, ENABLED_KEY, if on { "1" } else { "0" }).await
}

pub async fn last_sync(db: &Db) -> AppResult<Option<i64>> {
    Ok(settings::get_string(db, LAST_SYNC_KEY)
        .await?
        .and_then(|value| value.parse().ok()))
}

pub async fn set_last_sync(db: &Db, at: i64) -> AppResult<()> {
    settings::set_string(db, LAST_SYNC_KEY, &at.to_string()).await
}

/// Project names the user keeps out of the vault, one per line in the setting.
pub async fn excluded(db: &Db) -> AppResult<Vec<String>> {
    let stored = settings::get_string(db, EXCLUDED_KEY).await?;
    Ok(stored
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim().to_lowercase())
        .filter(|line| !line.is_empty())
        .collect())
}

pub async fn set_excluded(db: &Db, names: &[String]) -> AppResult<()> {
    settings::set_string(db, EXCLUDED_KEY, &names.join("\n")).await
}

pub async fn get(db: &Db, path: &str) -> AppResult<Option<FileState>> {
    Ok(sqlx::query_as("SELECT * FROM vault_files WHERE path = ?")
        .bind(path)
        .fetch_optional(&db.pool)
        .await?)
}

pub async fn put(db: &Db, state: &FileState) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO vault_files (path, kco_id, hash, app_hash, mtime, synced_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(path) DO UPDATE
            SET kco_id = excluded.kco_id, hash = excluded.hash, app_hash = excluded.app_hash,
                mtime = excluded.mtime, synced_at = excluded.synced_at",
    )
    .bind(&state.path)
    .bind(&state.kco_id)
    .bind(&state.hash)
    .bind(&state.app_hash)
    .bind(state.mtime)
    .bind(state.synced_at)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// A file that was moved or renamed in the vault keeps its row under the new path.
pub async fn forget(db: &Db, path: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM vault_files WHERE path = ?")
        .bind(path)
        .execute(&db.pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(path: &str, hash: &str) -> FileState {
        FileState {
            path: path.into(),
            kco_id: "id".into(),
            hash: hash.into(),
            app_hash: "app".into(),
            mtime: 1,
            synced_at: 2,
        }
    }

    #[tokio::test]
    async fn the_path_defaults_and_can_be_changed() {
        let db = Db::in_memory().await;
        assert_eq!(path(&db).await.expect("path"), DEFAULT_VAULT);
        set_path(&db, "  D:/Vault  ").await.expect("set");
        assert_eq!(path(&db).await.expect("path"), "D:/Vault");
    }

    #[tokio::test]
    async fn a_file_state_is_stored_replaced_and_forgotten() {
        let db = Db::in_memory().await;
        put(&db, &state("a.md", "one")).await.expect("put");
        put(&db, &state("a.md", "two")).await.expect("replace");
        assert_eq!(
            get(&db, "a.md").await.expect("get"),
            Some(state("a.md", "two"))
        );
        forget(&db, "a.md").await.expect("forget");
        assert_eq!(get(&db, "a.md").await.expect("get"), None);
    }

    #[tokio::test]
    async fn exclusions_are_trimmed_and_lowercased() {
        let db = Db::in_memory().await;
        set_excluded(&db, &[" Secret App ".into(), "".into()])
            .await
            .expect("set");
        assert_eq!(excluded(&db).await.expect("get"), ["secret app"]);
    }

    #[tokio::test]
    async fn sync_is_off_until_turned_on() {
        let db = Db::in_memory().await;
        assert!(!enabled(&db).await.expect("get"));
        set_enabled(&db, true).await.expect("set");
        assert!(enabled(&db).await.expect("get"));
    }

    #[tokio::test]
    async fn last_sync_is_none_until_set() {
        let db = Db::in_memory().await;
        assert_eq!(last_sync(&db).await.expect("get"), None);
        set_last_sync(&db, 99).await.expect("set");
        assert_eq!(last_sync(&db).await.expect("get"), Some(99));
    }
}
