use super::civil_date;
use super::task::{Priority, TaskStatus};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub const MAX_DURATION_MIN: i64 = 1440;
pub const DEFAULT_AVAILABLE_MIN: i64 = 6 * 60;
const MIN_AVAILABLE_MIN: i64 = 30;

/// When a task is planned: a local day, an optional time block on it and an estimate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct TaskPlan {
    pub planned_on: Option<String>,
    pub scheduled_at: Option<i64>,
    pub duration_min: Option<i64>,
}

impl TaskPlan {
    /// Checks the plan and stores the day in its canonical `YYYY-MM-DD` form.
    pub fn clean(self) -> AppResult<Self> {
        let planned_on = self
            .planned_on
            .as_deref()
            .map(civil_date::clean)
            .transpose()?;
        if self.scheduled_at.is_some() && planned_on.is_none() {
            return Err(AppError::Invalid("a scheduled task needs a planned day"));
        }
        if self.scheduled_at.is_some_and(|at| at < 0) {
            return Err(AppError::Invalid("the time block start is not valid"));
        }
        if self
            .duration_min
            .is_some_and(|minutes| !(1..=MAX_DURATION_MIN).contains(&minutes))
        {
            return Err(AppError::Invalid("the duration must be 1 to 1440 minutes"));
        }
        Ok(Self { planned_on, ..self })
    }
}

pub fn validate_available_minutes(minutes: i64) -> AppResult<()> {
    if (MIN_AVAILABLE_MIN..=MAX_DURATION_MIN).contains(&minutes) {
        Ok(())
    } else {
        Err(AppError::Invalid(
            "the hours you have must be between half an hour and 24 hours",
        ))
    }
}

/// A task with its project and plan, as the planner pages show it.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PlannedTask {
    pub id: String,
    pub title: String,
    pub project_id: String,
    pub project_name: String,
    pub project_color: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_at: Option<i64>,
    pub recurrence: Option<String>,
    pub planned_on: Option<String>,
    pub scheduled_at: Option<i64>,
    pub duration_min: Option<i64>,
}

/// A recorded focus or manual session, shown read-only in the week grid.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct FocusBlock {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub project_color: Option<String>,
    pub task_title: Option<String>,
    pub started_at: i64,
    pub ended_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(
        planned_on: Option<&str>,
        scheduled_at: Option<i64>,
        duration: Option<i64>,
    ) -> TaskPlan {
        TaskPlan {
            planned_on: planned_on.map(str::to_string),
            scheduled_at,
            duration_min: duration,
        }
    }

    #[test]
    fn a_full_plan_is_kept_with_a_canonical_day() {
        let cleaned = plan(Some(" 2026-11-10"), Some(1_000), Some(45))
            .clean()
            .expect("valid");
        assert_eq!(cleaned, plan(Some("2026-11-10"), Some(1_000), Some(45)));
        assert_eq!(
            TaskPlan::default().clean().expect("empty"),
            TaskPlan::default()
        );
    }

    #[test]
    fn bad_days_and_durations_are_refused() {
        for bad in [
            plan(Some("2026-02-30"), None, None),
            plan(Some("tomorrow"), None, None),
            plan(None, Some(1_000), Some(30)),
            plan(Some("2026-11-10"), Some(-1), Some(30)),
            plan(Some("2026-11-10"), None, Some(0)),
            plan(Some("2026-11-10"), None, Some(1441)),
        ] {
            assert!(matches!(bad.clean(), Err(AppError::Invalid(_))));
        }
        assert!(plan(None, None, Some(1440)).clean().is_ok());
        assert!(plan(None, None, Some(1)).clean().is_ok());
    }

    #[test]
    fn available_hours_are_bounded() {
        assert!(validate_available_minutes(DEFAULT_AVAILABLE_MIN).is_ok());
        assert!(validate_available_minutes(30).is_ok());
        assert!(validate_available_minutes(29).is_err());
        assert!(validate_available_minutes(1441).is_err());
    }
}
