use crate::error::{AppError, AppResult};
use serde::Serialize;

pub const DEFAULT_KEEP: i64 = 14;
const MAX_KEEP: i64 = 365;
const HOUR_MS: i64 = 60 * 60 * 1000;
const DAY_MS: i64 = 24 * HOUR_MS;

#[derive(Debug, Clone, Serialize)]
pub struct BackupFile {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupStatus {
    pub folder: String,
    /// false while the default folder next to the database is used
    pub custom_folder: bool,
    pub keep: i64,
    pub last_at: Option<i64>,
    pub last_error: Option<String>,
    /// newest first
    pub copies: Vec<BackupFile>,
}

pub fn validate_keep(keep: i64) -> AppResult<()> {
    if (1..=MAX_KEEP).contains(&keep) {
        Ok(())
    } else {
        Err(AppError::Invalid("keep 1 to 365 backups"))
    }
}

/// A day after the last good backup, retrying a failed one at most once an hour.
pub fn is_due(now: i64, last_at: Option<i64>, last_attempt_at: Option<i64>) -> bool {
    let stale = last_at.is_none_or(|at| now - at >= DAY_MS);
    let may_retry = last_attempt_at.is_none_or(|at| now - at >= HOUR_MS);
    stale && may_retry
}

/// The timestamp in a `<prefix><ms>.db` file name.
pub fn created_at(name: &str, prefix: &str) -> Option<i64> {
    name.strip_prefix(prefix)?.strip_suffix(".db")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_once_a_day_with_hourly_retries() {
        let now = 10 * DAY_MS;
        assert!(is_due(now, None, None));
        assert!(!is_due(now, Some(now - HOUR_MS), Some(now - HOUR_MS)));
        assert!(is_due(now, Some(now - DAY_MS), Some(now - DAY_MS)));
        assert!(
            !is_due(now, Some(now - 2 * DAY_MS), Some(now - 60_000)),
            "a failed attempt waits an hour"
        );
        assert!(is_due(now, Some(now - 2 * DAY_MS), Some(now - HOUR_MS)));
    }

    #[test]
    fn keep_has_bounds() {
        assert!(validate_keep(14).is_ok());
        assert!(validate_keep(0).is_err());
        assert!(validate_keep(366).is_err());
    }

    #[test]
    fn reads_the_time_from_a_file_name() {
        assert_eq!(
            created_at("daily-1700000000000.db", "daily-"),
            Some(1_700_000_000_000)
        );
        assert_eq!(created_at("daily-x.db", "daily-"), None);
        assert_eq!(created_at("pre-sync-1.db", "daily-"), None);
    }
}
