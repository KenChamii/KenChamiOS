use super::dates::{day_start_ms, day_string, days_of, iso_week, monday_of, parse_day};
use super::file::{Front, Region};
use crate::db::{Db, now_ms};
use crate::db::{daily_notes, docs, planner, projects, tasks, vault};
use crate::domain::project::{INBOX_ID, ProjectFilter};
use crate::domain::{DocKind, TaskStatus};
use crate::error::AppResult;
use std::collections::HashSet;

pub const APP_DIR: &str = "KenChamiOS";
const NOTE_LOOKBACK_DAYS: i64 = 60;
const PLAN_LOOKBACK_DAYS: i64 = 7;
const PLAN_LOOKAHEAD_DAYS: i64 = 14;
const DASHBOARD_TASKS: usize = 20;

/// Where a file's content comes from and, for two-way files, goes back to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Daily { day: String },
    Project { id: String },
    Dashboard,
    Review,
}

impl Target {
    /// Daily notes and project docs can be written in the vault; the rest only flow out.
    pub fn two_way(&self) -> bool {
        matches!(self, Self::Daily { .. } | Self::Project { .. })
    }
}

/// One app-owned file: its place, identity and what the app says should be in it.
#[derive(Debug, Clone)]
pub struct Spec {
    pub path: String,
    pub front: Front,
    pub title: String,
    pub regions: Vec<Region>,
    pub target: Target,
    /// the tasks a tick in this file may change
    pub tick_tasks: Vec<(String, bool)>,
}

fn front(id: &str, kind: &str) -> Front {
    Front {
        kco_id: id.to_string(),
        kco_type: kind.to_string(),
        updated: now_ms(),
    }
}

fn tick_line(id: &str, title: &str, done: bool) -> String {
    format!(
        "- [{}] {title} <!-- kco:task:{id} -->",
        if done { "x" } else { " " }
    )
}

/// The daily file for `day`: the note both ways, the planned tasks with ticks.
pub async fn daily(db: &Db, day: &str) -> AppResult<Spec> {
    let note = daily_notes::get(db, day).await?.map(|n| n.body);
    let planned = planner::planned(db, day, day).await?;
    let mut regions = vec![Region {
        name: "note",
        heading: "Note",
        content: note.unwrap_or_default().trim().to_string(),
    }];
    let ticks: Vec<(String, bool)> = planned
        .iter()
        .map(|task| (task.id.clone(), task.status == TaskStatus::Done))
        .collect();
    if !planned.is_empty() {
        let lines: Vec<String> = planned
            .iter()
            .map(|task| tick_line(&task.id, &task.title, task.status == TaskStatus::Done))
            .collect();
        regions.push(Region {
            name: "tasks",
            heading: "Planned",
            content: lines.join("\n"),
        });
    }
    Ok(Spec {
        path: format!("{APP_DIR}/Daily/{day}.md"),
        front: front(&format!("daily:{day}"), "daily"),
        title: day.to_string(),
        regions,
        target: Target::Daily {
            day: day.to_string(),
        },
        tick_tasks: ticks,
    })
}

/// Idea and plan of every project that is not archived, the Inbox or excluded.
pub async fn project_specs(db: &Db) -> AppResult<Vec<Spec>> {
    let excluded = vault::excluded(db).await?;
    let list = projects::list(db, &ProjectFilter::default()).await?;
    let mut taken = HashSet::new();
    let mut specs = Vec::new();
    for summary in list {
        let project = summary.project;
        if project.id == INBOX_ID
            || project.archived
            || excluded.contains(&project.name.to_lowercase())
        {
            continue;
        }
        let mut file = project.slug.clone();
        if !taken.insert(file.clone()) {
            file = format!("{file}-{}", project.id.chars().take(6).collect::<String>());
        }
        let idea = doc_body(db, &project.id, DocKind::Idea).await?;
        let plan = doc_body(db, &project.id, DocKind::Plan).await?;
        specs.push(Spec {
            path: format!("{APP_DIR}/Projects/{file}.md"),
            front: front(&format!("project:{}", project.id), "project"),
            title: project.name.clone(),
            regions: vec![
                Region {
                    name: "idea",
                    heading: "Idea",
                    content: idea,
                },
                Region {
                    name: "plan",
                    heading: "Plan",
                    content: plan,
                },
            ],
            target: Target::Project {
                id: project.id.clone(),
            },
            tick_tasks: Vec::new(),
        });
    }
    Ok(specs)
}

/// Open projects with progress, and what is planned for today.
pub async fn dashboard(db: &Db, today: &str) -> AppResult<Spec> {
    let excluded = vault::excluded(db).await?;
    let list = projects::list(db, &ProjectFilter::default()).await?;
    let mut lines = Vec::new();
    for summary in &list {
        let project = &summary.project;
        if project.id == INBOX_ID || excluded.contains(&project.name.to_lowercase()) {
            continue;
        }
        lines.push(format!(
            "- {} ({}): {}/{} tasks done",
            project.name,
            project.stage.as_str(),
            summary.tasks_done,
            summary.tasks_total
        ));
    }
    let planned = planner::planned(db, today, today).await?;
    let today_lines: Vec<String> = planned
        .iter()
        .take(DASHBOARD_TASKS)
        .map(|task| tick_line(&task.id, &task.title, task.status == TaskStatus::Done))
        .collect();
    let inbox_open = tasks::list(db, INBOX_ID)
        .await?
        .iter()
        .filter(|task| task.status != TaskStatus::Done)
        .count();
    Ok(Spec {
        path: format!("{APP_DIR}/Dashboard.md"),
        front: front("dashboard", "dashboard"),
        title: "Dashboard".into(),
        regions: vec![
            Region {
                name: "today",
                heading: "Today",
                content: or_none(&today_lines.join("\n"), "Nothing planned."),
            },
            Region {
                name: "projects",
                heading: "Projects",
                content: or_none(&lines.join("\n"), "No projects yet."),
            },
            Region {
                name: "inbox",
                heading: "Inbox",
                content: format!("{inbox_open} open"),
            },
        ],
        target: Target::Dashboard,
        tick_tasks: Vec::new(),
    })
}

fn or_none(text: &str, empty: &str) -> String {
    if text.is_empty() {
        empty.to_string()
    } else {
        text.to_string()
    }
}

/// What was finished in the ISO week that starts on `monday` (a day count).
pub async fn review(db: &Db, monday: i64) -> AppResult<Spec> {
    let (year, week) = iso_week(monday);
    let done = planner::completed(db, day_start_ms(monday), day_start_ms(monday + 7)).await?;
    let lines: Vec<String> = done
        .iter()
        .map(|task| format!("- {} ({})", task.title, task.project_name))
        .collect();
    let id = format!("review:{year}-W{week:02}");
    Ok(Spec {
        path: format!("{APP_DIR}/Reviews/{year}-W{week:02}.md"),
        front: front(&id, "review"),
        title: format!("Week {week}, {year}"),
        regions: vec![Region {
            name: "done",
            heading: "Finished",
            content: or_none(&lines.join("\n"), "Nothing finished yet."),
        }],
        target: Target::Review,
        tick_tasks: Vec::new(),
    })
}

/// Every file the app owns right now. `extra_days` are daily files already in the vault.
pub async fn all(db: &Db, now: i64, extra_days: &[String]) -> AppResult<Vec<Spec>> {
    let today = days_of(now);
    let today_text = day_string(today);
    let mut days: Vec<String> =
        daily_notes::days_with_notes(db, &day_string(today - NOTE_LOOKBACK_DAYS), &today_text)
            .await?;
    let planned = planner::planned(
        db,
        &day_string(today - PLAN_LOOKBACK_DAYS),
        &day_string(today + PLAN_LOOKAHEAD_DAYS),
    )
    .await?;
    days.extend(planned.into_iter().filter_map(|task| task.planned_on));
    days.extend(extra_days.iter().cloned());
    days.push(today_text.clone());
    days.sort();
    days.dedup();
    days.retain(|day| parse_day(day).is_some());

    let mut specs = Vec::new();
    for day in &days {
        specs.push(daily(db, day).await?);
    }
    specs.extend(project_specs(db).await?);
    specs.push(dashboard(db, &today_text).await?);
    let monday = monday_of(today);
    specs.push(review(db, monday).await?);
    specs.push(review(db, monday - 7).await?);
    Ok(specs)
}

async fn doc_body(db: &Db, project_id: &str, kind: DocKind) -> AppResult<String> {
    Ok(docs::get(db, project_id, kind)
        .await?
        .map(|doc| doc.body_md.trim().to_string())
        .unwrap_or_default())
}
