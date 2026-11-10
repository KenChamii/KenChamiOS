use super::civil_date;
use serde::Serialize;
use std::collections::HashSet;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
pub const RECENT_DAYS: i64 = 7;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HabitView {
    pub id: String,
    pub name: String,
    /// days in a row up to today, or up to yesterday while today is still open
    pub streak: i64,
    pub checked_today: bool,
    /// the last seven days, oldest first, today last
    pub recent: Vec<bool>,
}

/// One check-in: the habit and its local `YYYY-MM-DD` day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct HabitCheck {
    pub habit_id: String,
    pub day: String,
}

// days since the epoch, so consecutive dates differ by one
fn day_number(day: &str) -> Option<i64> {
    civil_date::parse(day).map(|ms| ms / DAY_MS)
}

/// Streak, today's state and the last week for one habit; `days` are its checked `YYYY-MM-DD` dates.
pub fn summarize(id: String, name: String, days: &[String], today: &str) -> Option<HabitView> {
    let today = day_number(today)?;
    let checked: HashSet<i64> = days.iter().filter_map(|d| day_number(d)).collect();
    let checked_today = checked.contains(&today);
    let mut cursor = if checked_today { today } else { today - 1 };
    let mut streak = 0;
    while checked.contains(&cursor) {
        streak += 1;
        cursor -= 1;
    }
    let recent = (0..RECENT_DAYS)
        .rev()
        .map(|back| checked.contains(&(today - back)))
        .collect();
    Some(HabitView {
        id,
        name,
        streak,
        checked_today,
        recent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(days: &[&str], today: &str) -> HabitView {
        let days: Vec<String> = days.iter().map(|d| d.to_string()).collect();
        summarize("h".into(), "Read".into(), &days, today).expect("valid today")
    }

    #[test]
    fn counts_the_run_that_ends_today() {
        let habit = view(&["2026-11-08", "2026-11-09", "2026-11-10"], "2026-11-10");
        assert_eq!((habit.streak, habit.checked_today), (3, true));
        assert_eq!(habit.recent, [false, false, false, false, true, true, true]);
    }

    #[test]
    fn today_not_yet_checked_keeps_yesterdays_streak() {
        let habit = view(&["2026-11-08", "2026-11-09"], "2026-11-10");
        assert_eq!((habit.streak, habit.checked_today), (2, false));
    }

    #[test]
    fn a_missed_day_breaks_the_streak() {
        assert_eq!(view(&["2026-11-07", "2026-11-08"], "2026-11-10").streak, 0);
        assert_eq!(
            view(
                &["2026-11-06", "2026-11-08", "2026-11-09", "2026-11-10"],
                "2026-11-10"
            )
            .streak,
            3
        );
    }

    #[test]
    fn streaks_run_across_month_and_year_ends() {
        let habit = view(&["2026-12-30", "2026-12-31", "2027-01-01"], "2027-01-01");
        assert_eq!(habit.streak, 3);
        let leap = view(&["2024-02-28", "2024-02-29", "2024-03-01"], "2024-03-01");
        assert_eq!(leap.streak, 3);
    }

    #[test]
    fn no_checks_and_unreadable_days() {
        assert_eq!(view(&[], "2026-11-10").streak, 0);
        assert_eq!(view(&["garbage", "2026-11-10"], "2026-11-10").streak, 1);
        assert!(summarize("h".into(), "Read".into(), &[], "not a day").is_none());
    }
}
