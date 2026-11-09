use super::export::format_date;
use super::{SavePoint, Task};
use crate::domain::asset::{AssetKind, ProjectAsset};
use crate::domain::time_entry::{EntryKind, TimeEntry};
use serde::Serialize;

/// Folder inside the export folder that holds the copied screenshots.
pub const IMAGES_DIR: &str = "images";

#[derive(Debug, Default, Serialize)]
pub struct PortfolioCtx {
    pub features: Vec<String>,
    pub timeline: Vec<TimelineCtx>,
    pub time: TimeCtx,
    pub screenshots: Vec<ScreenshotCtx>,
    pub links: Vec<LinkCtx>,
}

#[derive(Debug, Serialize)]
pub struct TimelineCtx {
    pub date: String,
    pub stage: &'static str,
    pub note: String,
}

/// Empty `total` when nothing was logged, so templates can skip the section.
#[derive(Debug, Default, Serialize)]
pub struct TimeCtx {
    pub total: String,
    pub sessions: usize,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotCtx {
    /// relative to the exported markdown file
    pub path: String,
    pub caption: String,
    #[serde(skip)]
    pub stored_name: String,
}

#[derive(Debug, Serialize)]
pub struct LinkCtx {
    pub url: String,
    pub caption: String,
}

impl PortfolioCtx {
    /// `done_tasks` are the finished top-level tasks; `save_points` come newest first.
    pub fn build(
        done_tasks: &[Task],
        save_points: &[SavePoint],
        entries: &[TimeEntry],
        assets: &[ProjectAsset],
    ) -> Self {
        let worked: i64 = entries.iter().map(|e| e.duration_s).sum();
        Self {
            features: done_tasks.iter().map(|t| t.title.clone()).collect(),
            timeline: save_points
                .iter()
                .rev()
                .map(|point| TimelineCtx {
                    date: format_date(point.created_at),
                    stage: point.stage.as_str(),
                    note: point.note.clone(),
                })
                .collect(),
            time: TimeCtx {
                total: if worked > 0 {
                    format_hours(worked)
                } else {
                    String::new()
                },
                sessions: entries
                    .iter()
                    .filter(|e| e.kind == EntryKind::Focus)
                    .count(),
            },
            screenshots: assets
                .iter()
                .filter(|a| a.kind == AssetKind::Screenshot)
                .map(|a| ScreenshotCtx {
                    path: format!("{IMAGES_DIR}/{}", a.path_or_url),
                    caption: a.caption.clone(),
                    stored_name: a.path_or_url.clone(),
                })
                .collect(),
            links: assets
                .iter()
                .filter(|a| a.kind == AssetKind::Link)
                .map(|a| LinkCtx {
                    url: a.path_or_url.clone(),
                    caption: a.caption.clone(),
                })
                .collect(),
        }
    }

    /// Made-up data for the template preview in settings.
    pub fn sample() -> Self {
        Self {
            features: vec!["Login form".into(), "Dark mode".into()],
            timeline: vec![
                TimelineCtx {
                    date: "2026-10-01".into(),
                    stage: "plan",
                    note: "Scoped the first version.".into(),
                },
                TimelineCtx {
                    date: "2026-11-02".into(),
                    stage: "done",
                    note: "Shipped.".into(),
                },
            ],
            time: TimeCtx {
                total: "12h 30m".into(),
                sessions: 20,
            },
            screenshots: vec![ScreenshotCtx {
                path: format!("{IMAGES_DIR}/home.png"),
                caption: "Home screen".into(),
                stored_name: "home.png".into(),
            }],
            links: vec![LinkCtx {
                url: "https://example.com".into(),
                caption: "Demo".into(),
            }],
        }
    }
}

/// Seconds as "12h 30m", or "45m" under an hour.
fn format_hours(seconds: i64) -> String {
    let minutes = (seconds + 30) / 60;
    match minutes / 60 {
        0 => format!("{minutes}m"),
        hours => format!("{hours}h {:02}m", minutes % 60),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hours_are_rounded_to_the_minute() {
        assert_eq!(format_hours(29), "0m");
        assert_eq!(format_hours(90), "2m");
        assert_eq!(format_hours(45 * 60), "45m");
        assert_eq!(format_hours(3600), "1h 00m");
        assert_eq!(format_hours(12 * 3600 + 30 * 60), "12h 30m");
    }
}
