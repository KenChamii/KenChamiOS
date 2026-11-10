use super::media::MediaType;
use super::task::{Priority, TaskStatus};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

const MAX_BUCKETS: usize = 400;

/// A task that is due today or overdue, with the project it belongs to.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DueTask {
    pub id: String,
    pub title: String,
    pub project_id: String,
    pub project_name: String,
    pub due_at: i64,
    pub priority: Priority,
    pub status: TaskStatus,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineGroup {
    Projects,
    Focus,
    Media,
}

impl TimelineGroup {
    /// The part of an activity kind before the dot that belongs to this group.
    pub fn prefixes(self) -> &'static [&'static str] {
        match self {
            Self::Projects => &[
                "project",
                "task",
                "checklist",
                "doc",
                "save_point",
                "prompt",
                "asset",
            ],
            Self::Focus => &["time"],
            Self::Media => &["media"],
        }
    }

    pub fn of_kind(kind: &str) -> Option<Self> {
        let prefix = kind.split('.').next()?;
        [Self::Projects, Self::Focus, Self::Media]
            .into_iter()
            .find(|group| group.prefixes().contains(&prefix))
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ActivityRow {
    pub id: String,
    pub kind: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimelineItem {
    pub id: String,
    pub kind: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub at: i64,
    pub group: Option<TimelineGroup>,
}

impl From<ActivityRow> for TimelineItem {
    fn from(row: ActivityRow) -> Self {
        Self {
            group: TimelineGroup::of_kind(&row.kind),
            id: row.id,
            kind: row.kind,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            summary: row.summary,
            at: row.at,
        }
    }
}

/// The period to summarise and where its chart buckets start. All times are unix ms and come
/// from the UI, which knows the local time zone; `to` and the last boundary are exclusive.
#[derive(Debug, Clone, Deserialize)]
pub struct RecapRange {
    pub from: i64,
    pub to: i64,
    pub buckets: Vec<i64>,
}

impl RecapRange {
    pub fn validate(&self) -> AppResult<()> {
        if self.from >= self.to {
            return Err(AppError::Invalid("the period must end after it starts"));
        }
        if self.buckets.len() > MAX_BUCKETS
            || self.buckets.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(AppError::Invalid("the chart boundaries are not valid"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectFocus {
    pub project_id: String,
    pub name: String,
    pub color: Option<String>,
    pub seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MediaTypeStat {
    pub media_type: MediaType,
    /// episodes, chapters, hours or listens logged in the period
    pub units: i64,
    pub completed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GenreCount {
    pub genre: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecapView {
    pub focus_seconds: i64,
    pub focus_by_bucket: Vec<i64>,
    pub focus_by_project: Vec<ProjectFocus>,
    pub media: Vec<MediaTypeStat>,
    pub top_genres: Vec<GenreCount>,
    pub stage_changes: i64,
    pub projects_finished: Vec<String>,
}

/// One focus or manual entry with its project, as the recap reads it.
#[derive(Debug, Clone, FromRow)]
pub struct FocusRow {
    pub project_id: String,
    pub project_name: String,
    pub project_color: Option<String>,
    pub started_at: i64,
    pub duration_s: i64,
}

/// Seconds in each bucket `[boundaries[i], boundaries[i + 1])`; rows outside every bucket are ignored.
pub fn bucket_totals(boundaries: &[i64], rows: &[FocusRow]) -> Vec<i64> {
    let mut totals = vec![0; boundaries.len().saturating_sub(1)];
    for row in rows {
        let after = boundaries.partition_point(|boundary| *boundary <= row.started_at);
        if after > 0 && after < boundaries.len() {
            totals[after - 1] += row.duration_s;
        }
    }
    totals
}

/// Seconds per project, largest first.
pub fn project_totals(rows: &[FocusRow]) -> Vec<ProjectFocus> {
    let mut totals: Vec<ProjectFocus> = Vec::new();
    for row in rows {
        match totals.iter_mut().find(|t| t.project_id == row.project_id) {
            Some(total) => total.seconds += row.duration_s,
            None => totals.push(ProjectFocus {
                project_id: row.project_id.clone(),
                name: row.project_name.clone(),
                color: row.project_color.clone(),
                seconds: row.duration_s,
            }),
        }
    }
    totals.sort_by(|a, b| b.seconds.cmp(&a.seconds).then_with(|| a.name.cmp(&b.name)));
    totals
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(project: &str, started_at: i64, duration_s: i64) -> FocusRow {
        FocusRow {
            project_id: project.into(),
            project_name: project.to_uppercase(),
            project_color: None,
            started_at,
            duration_s,
        }
    }

    #[test]
    fn buckets_include_their_start_and_exclude_their_end() {
        let boundaries = [100, 200, 300];
        let rows = [
            row("a", 99, 1),
            row("a", 100, 10),
            row("a", 199, 20),
            row("a", 200, 40),
            row("a", 299, 80),
            row("a", 300, 160),
        ];

        assert_eq!(bucket_totals(&boundaries, &rows), [30, 120]);
    }

    #[test]
    fn no_boundaries_or_one_boundary_give_no_buckets() {
        let rows = [row("a", 5, 10)];

        assert!(bucket_totals(&[], &rows).is_empty());
        assert!(bucket_totals(&[5], &rows).is_empty());
    }

    #[test]
    fn project_totals_are_summed_and_sorted() {
        let rows = [row("b", 1, 10), row("a", 2, 30), row("b", 3, 25)];

        let totals = project_totals(&rows);

        assert_eq!(totals[0].project_id, "b");
        assert_eq!(totals[0].seconds, 35);
        assert_eq!(totals[1].project_id, "a");
        assert_eq!(totals[1].seconds, 30);
    }

    #[test]
    fn project_totals_with_equal_seconds_sort_by_name() {
        let rows = [row("b", 1, 10), row("a", 2, 10)];

        let names: Vec<_> = project_totals(&rows).into_iter().map(|p| p.name).collect();

        assert_eq!(names, ["A", "B"]);
    }

    #[test]
    fn ranges_must_be_ordered_and_bounded() {
        let range = |from, to, buckets: Vec<i64>| RecapRange { from, to, buckets };

        assert!(range(0, 10, vec![0, 5, 10]).validate().is_ok());
        assert!(range(10, 10, vec![]).validate().is_err());
        assert!(range(0, 10, vec![0, 5, 5]).validate().is_err());
        assert!(range(0, 10, vec![5, 0]).validate().is_err());
        assert!(range(0, 10, (0..401).collect()).validate().is_err());
    }

    #[test]
    fn kinds_map_to_their_group() {
        assert_eq!(
            TimelineGroup::of_kind("task.moved"),
            Some(TimelineGroup::Projects)
        );
        assert_eq!(
            TimelineGroup::of_kind("save_point.created"),
            Some(TimelineGroup::Projects)
        );
        assert_eq!(
            TimelineGroup::of_kind("time.logged"),
            Some(TimelineGroup::Focus)
        );
        assert_eq!(
            TimelineGroup::of_kind("media.progress"),
            Some(TimelineGroup::Media)
        );
        assert_eq!(TimelineGroup::of_kind("mystery.thing"), None);
    }
}
