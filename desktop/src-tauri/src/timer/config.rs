use super::machine::TimerConfig;
use crate::db::{Db, settings};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

const MAX_MINUTES: u32 = 180;

/// Phase lengths in minutes, as shown in settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerSettings {
    pub focus_min: u32,
    pub break_min: u32,
    pub long_break_min: u32,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            focus_min: 25,
            break_min: 5,
            long_break_min: 15,
        }
    }
}

impl TimerSettings {
    pub fn config(self) -> TimerConfig {
        TimerConfig {
            focus_s: self.focus_min * 60,
            break_s: self.break_min * 60,
            long_break_s: self.long_break_min * 60,
        }
    }

    fn validate(self) -> AppResult<()> {
        let valid = [self.focus_min, self.break_min, self.long_break_min]
            .iter()
            .all(|minutes| (1..=MAX_MINUTES).contains(minutes));
        if valid {
            Ok(())
        } else {
            Err(AppError::Invalid("timer lengths must be 1 to 180 minutes"))
        }
    }
}

const FOCUS_KEY: &str = "timer.focus_minutes";
const BREAK_KEY: &str = "timer.break_minutes";
const LONG_BREAK_KEY: &str = "timer.long_break_minutes";

pub async fn load(db: &Db) -> AppResult<TimerSettings> {
    let defaults = TimerSettings::default();
    Ok(TimerSettings {
        focus_min: read(db, FOCUS_KEY, defaults.focus_min).await?,
        break_min: read(db, BREAK_KEY, defaults.break_min).await?,
        long_break_min: read(db, LONG_BREAK_KEY, defaults.long_break_min).await?,
    })
}

pub async fn save(db: &Db, settings: TimerSettings) -> AppResult<TimerSettings> {
    settings.validate()?;
    write(db, FOCUS_KEY, settings.focus_min).await?;
    write(db, BREAK_KEY, settings.break_min).await?;
    write(db, LONG_BREAK_KEY, settings.long_break_min).await?;
    Ok(settings)
}

// a stored value that is not a valid length falls back to the default
async fn read(db: &Db, key: &str, default: u32) -> AppResult<u32> {
    let stored = settings::get_string(db, key).await?;
    Ok(stored
        .and_then(|text| text.parse::<u32>().ok())
        .filter(|minutes| (1..=MAX_MINUTES).contains(minutes))
        .unwrap_or(default))
}

async fn write(db: &Db, key: &str, minutes: u32) -> AppResult<()> {
    settings::set_string(db, key, &minutes.to_string()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn defaults_until_saved_then_round_trips() {
        let db = Db::in_memory().await;
        assert_eq!(load(&db).await.expect("load"), TimerSettings::default());

        let custom = TimerSettings {
            focus_min: 50,
            break_min: 10,
            long_break_min: 30,
        };
        save(&db, custom).await.expect("save");

        assert_eq!(load(&db).await.expect("load"), custom);
        assert_eq!(custom.config().focus_s, 3000);
    }

    #[tokio::test]
    async fn out_of_range_lengths_are_rejected() {
        let db = Db::in_memory().await;
        for focus_min in [0, 181] {
            let bad = TimerSettings {
                focus_min,
                ..TimerSettings::default()
            };
            assert!(matches!(save(&db, bad).await, Err(AppError::Invalid(_))));
        }
        assert_eq!(load(&db).await.expect("load"), TimerSettings::default());
    }

    #[tokio::test]
    async fn a_corrupt_stored_value_falls_back_to_the_default() {
        let db = Db::in_memory().await;
        settings::set_string(&db, FOCUS_KEY, "soon")
            .await
            .expect("set");

        assert_eq!(load(&db).await.expect("load").focus_min, 25);
    }
}
