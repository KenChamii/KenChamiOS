use super::project::Stage;
use crate::error::{AppError, AppResult};
use serde::Serialize;

pub const DEFAULT_IDLE_DAYS: i64 = 7;
const MAX_IDLE_DAYS: i64 = 365;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub fn validate_idle_days(days: i64) -> AppResult<()> {
    if (1..=MAX_IDLE_DAYS).contains(&days) {
        Ok(())
    } else {
        Err(AppError::Invalid(
            "the reminder delay must be 1 to 365 days",
        ))
    }
}

/// How long a project has been quiet, as the aggregate query reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectQuiet {
    pub project_id: String,
    pub name: String,
    pub stage: Stage,
    /// newest activity of the project, its tasks, docs, time and assets; save points do not count
    pub last_work_at: i64,
    pub last_save_point_at: Option<i64>,
    pub snoozed_until: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdleProject {
    pub project_id: String,
    pub name: String,
    pub stage: Stage,
    pub idle_days: i64,
    pub last_save_point_at: Option<i64>,
}

/// Projects in an active stage whose work stopped `idle_days` ago or more and was never saved
/// since, longest quiet first. A snoozed project stays hidden until its snooze ends.
pub fn idle_projects(rows: &[ProjectQuiet], now: i64, idle_days: i64) -> Vec<IdleProject> {
    let mut idle: Vec<IdleProject> = rows
        .iter()
        .filter(|row| matches!(row.stage, Stage::Plan | Stage::Build | Stage::Review))
        .filter(|row| now - row.last_work_at >= idle_days * DAY_MS)
        .filter(|row| {
            row.last_save_point_at
                .is_none_or(|at| at < row.last_work_at)
        })
        .filter(|row| row.snoozed_until.is_none_or(|until| until <= now))
        .map(|row| IdleProject {
            project_id: row.project_id.clone(),
            name: row.name.clone(),
            stage: row.stage,
            idle_days: (now - row.last_work_at) / DAY_MS,
            last_save_point_at: row.last_save_point_at,
        })
        .collect();
    idle.sort_by(|a, b| {
        b.idle_days
            .cmp(&a.idle_days)
            .then(a.name.cmp(&b.name))
            .then(a.project_id.cmp(&b.project_id))
    });
    idle
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 100 * DAY_MS;

    fn quiet(id: &str, stage: Stage, worked_days_ago: i64) -> ProjectQuiet {
        ProjectQuiet {
            project_id: id.to_string(),
            name: format!("Project {id}"),
            stage,
            last_work_at: NOW - worked_days_ago * DAY_MS,
            last_save_point_at: None,
            snoozed_until: None,
        }
    }

    fn ids(rows: &[ProjectQuiet], days: i64) -> Vec<String> {
        idle_projects(rows, NOW, days)
            .into_iter()
            .map(|p| p.project_id)
            .collect()
    }

    #[test]
    fn only_active_stages_are_suggested() {
        let rows = [
            quiet("idea", Stage::Idea, 30),
            quiet("plan", Stage::Plan, 30),
            quiet("build", Stage::Build, 30),
            quiet("review", Stage::Review, 30),
            quiet("done", Stage::Done, 30),
        ];

        assert_eq!(ids(&rows, 7), ["build", "plan", "review"]);
    }

    #[test]
    fn the_delay_is_inclusive_and_configurable() {
        let rows = [quiet("a", Stage::Build, 7), quiet("b", Stage::Build, 6)];

        assert_eq!(ids(&rows, 7), ["a"]);
        assert_eq!(ids(&rows, 6), ["a", "b"]);
        assert!(ids(&rows, 8).is_empty());
    }

    #[test]
    fn a_save_point_after_the_last_work_silences_the_reminder() {
        let mut saved = quiet("saved", Stage::Build, 20);
        saved.last_save_point_at = Some(NOW - 19 * DAY_MS);
        let mut stale = quiet("stale", Stage::Build, 20);
        stale.last_save_point_at = Some(NOW - 25 * DAY_MS);

        assert_eq!(ids(&[saved, stale], 7), ["stale"]);
    }

    #[test]
    fn a_snoozed_project_returns_when_the_snooze_ends() {
        let mut snoozed = quiet("a", Stage::Build, 20);
        snoozed.snoozed_until = Some(NOW + 1);
        assert!(ids(std::slice::from_ref(&snoozed), 7).is_empty());

        snoozed.snoozed_until = Some(NOW);
        assert_eq!(ids(&[snoozed], 7), ["a"]);
    }

    #[test]
    fn the_longest_quiet_comes_first_and_ties_are_stable() {
        let rows = [
            quiet("c", Stage::Build, 10),
            quiet("b", Stage::Build, 30),
            quiet("a", Stage::Build, 10),
        ];

        assert_eq!(ids(&rows, 7), ["b", "a", "c"]);
    }

    #[test]
    fn the_report_counts_whole_days() {
        let mut row = quiet("a", Stage::Build, 9);
        row.last_work_at -= DAY_MS / 2;

        let idle = idle_projects(&[row], NOW, 7);

        assert_eq!(idle[0].idle_days, 9);
    }

    #[test]
    fn the_delay_must_be_one_to_365_days() {
        for good in [1, 7, 365] {
            assert!(validate_idle_days(good).is_ok(), "{good}");
        }
        for bad in [0, -1, 366] {
            assert!(validate_idle_days(bad).is_err(), "{bad}");
        }
    }
}
