use super::process;
use crate::db::{Db, jobs, planner, settings, vault};
use crate::error::{AppError, AppResult};
use crate::mcp::tools;
use crate::vault::dates::{day_start_ms, day_string, days_of, monday_of};
use crate::vault::file::{self, Front};
use crate::workflow::safe_path::resolve_inside;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(300);
const MORNING_FROM_HOUR_UTC: i64 = 5;
const SUNDAY: i64 = 6;
const MONDAY: i64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BriefKind {
    Morning,
    Review,
    Jobs,
}

impl BriefKind {
    pub const ALL: [Self; 3] = [Self::Morning, Self::Review, Self::Jobs];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Morning => "morning",
            Self::Review => "review",
            Self::Jobs => "jobs",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Morning => "Morning brief",
            Self::Review => "Weekly review",
            Self::Jobs => "Job search summary",
        }
    }

    fn ask(self) -> &'static str {
        match self {
            Self::Morning => {
                "Write a short morning brief: what matters today, what is overdue, and the first thing to start."
            }
            Self::Review => {
                "Write a weekly review: what got done, what slipped, and three things for next week."
            }
            Self::Jobs => {
                "Write a job search summary: where each application stands, who to follow up with, and the next three actions."
            }
        }
    }
}

fn enabled_key(kind: BriefKind) -> String {
    format!("briefs.enabled.{}", kind.as_str())
}

fn last_key(kind: BriefKind) -> String {
    format!("briefs.last.{}", kind.as_str())
}

pub async fn is_enabled(db: &Db, kind: BriefKind) -> AppResult<bool> {
    Ok(settings::get_string(db, &enabled_key(kind))
        .await?
        .as_deref()
        == Some("1"))
}

pub async fn set_enabled(db: &Db, kind: BriefKind, on: bool) -> AppResult<()> {
    settings::set_string(db, &enabled_key(kind), if on { "1" } else { "0" }).await
}

/// Which kind of brief is due at `now`, and has not been written for today yet. Days and hours
/// are UTC, because the core has no time zone.
pub async fn due(db: &Db, now: i64) -> AppResult<Vec<BriefKind>> {
    let today = days_of(now);
    let hour = (now.rem_euclid(86_400_000)) / 3_600_000;
    let weekday = (today + 3).rem_euclid(7);
    let mut due = Vec::new();
    for kind in BriefKind::ALL {
        let on_time = match kind {
            BriefKind::Morning => hour >= MORNING_FROM_HOUR_UTC,
            BriefKind::Review => weekday == SUNDAY,
            BriefKind::Jobs => weekday == MONDAY,
        };
        let done = settings::get_string(db, &last_key(kind)).await?;
        if on_time && is_enabled(db, kind).await? && done.as_deref() != Some(&day_string(today)) {
            due.push(kind);
        }
    }
    Ok(due)
}

/// What the brief is written from: plain facts out of the app, no secrets.
async fn facts(db: &Db, kind: BriefKind, now: i64) -> AppResult<String> {
    match kind {
        BriefKind::Morning => Ok(tools::call(db, "read_today", &json!({}), now).await?.text),
        BriefKind::Jobs => {
            let list = tools::call(db, "list_jobs", &json!({}), now).await?.text;
            let today = tools::call(db, "read_today", &json!({}), now).await?.text;
            Ok(format!("{list}\n\n{today}"))
        }
        BriefKind::Review => {
            let monday = monday_of(days_of(now));
            let done = planner::completed(db, day_start_ms(monday), now + 1).await?;
            let lines: Vec<String> = done
                .iter()
                .map(|t| format!("- {} ({})", t.title, t.project_name))
                .collect();
            let finished = if lines.is_empty() {
                "Nothing finished yet.".to_string()
            } else {
                lines.join("\n")
            };
            let jobs = jobs::list(db, now).await?.len();
            Ok(format!(
                "Finished this week:\n{finished}\n\nApplications tracked: {jobs}."
            ))
        }
    }
}

fn prompt(kind: BriefKind, facts: &str) -> String {
    format!(
        "You write notes for a personal knowledge base. {}\n\
         Keep it under 200 words, in markdown, with no preamble.\n\n\
         Facts from the app:\n{facts}\n",
        kind.ask()
    )
}

fn command(cwd: &Path) -> std::process::Command {
    let mut cmd = process::command("claude", cwd);
    // no tools, no mcp, no settings files: it only writes text from the facts it was given
    cmd.args([
        "-p",
        "--output-format",
        "text",
        "--restricted",
        "--tools",
        "",
    ])
    .args(["--permission-prompts", "none"])
    .args([
        "--strict-mcp-config",
        "--disable-slash-commands",
        "--no-session-persistence",
    ]);
    cmd
}

/// Writes one brief into the vault and remembers the day. Blocking work runs off the UI thread.
pub async fn run(db: &Db, data_dir: &Path, kind: BriefKind, now: i64) -> AppResult<String> {
    let root = vault::path(db).await?;
    if !Path::new(&root).is_dir() {
        return Err(AppError::Invalid("the vault folder does not exist"));
    }
    let text = prompt(kind, &facts(db, kind, now).await?);
    let cwd = data_dir.to_path_buf();
    let answer =
        tauri::async_runtime::spawn_blocking(move || process::run(command(&cwd), &text, TIMEOUT))
            .await
            .map_err(|e| AppError::Agent(e.to_string()))??;
    if !answer.success || answer.stdout.trim().is_empty() {
        return Err(AppError::Agent(format!(
            "Claude Code gave no brief: {}",
            answer.stderr.trim()
        )));
    }
    let day = day_string(days_of(now));
    let relative = format!("KenChamiOS/Briefs/{day} {}.md", kind.as_str());
    let path = resolve_inside(&root, &relative)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let front = Front {
        kco_id: format!("brief:{}:{day}", kind.as_str()),
        kco_type: "brief".into(),
        updated: now,
    };
    let body = format!("# {} {day}\n\n{}\n", kind.title(), answer.stdout.trim());
    fs::write(&path, file::assemble(&front, &body))?;
    settings::set_string(db, &last_key(kind), &day).await?;
    Ok(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-16 is a Monday
    const MONDAY_NOON: i64 = 20_773 * 86_400_000 + 12 * 3_600_000;

    #[tokio::test]
    async fn nothing_is_due_until_turned_on() {
        let db = Db::in_memory().await;
        assert!(due(&db, MONDAY_NOON).await.expect("due").is_empty());
    }

    #[tokio::test]
    async fn the_morning_brief_comes_after_five_and_once_a_day() {
        let db = Db::in_memory().await;
        set_enabled(&db, BriefKind::Morning, true)
            .await
            .expect("on");

        let early = 20_773 * 86_400_000 + 3_600_000;
        assert!(due(&db, early).await.expect("early").is_empty());
        assert_eq!(
            due(&db, MONDAY_NOON).await.expect("noon"),
            [BriefKind::Morning]
        );

        settings::set_string(&db, &last_key(BriefKind::Morning), &day_string(20_773))
            .await
            .expect("done");
        assert!(due(&db, MONDAY_NOON).await.expect("again").is_empty());
    }

    #[tokio::test]
    async fn the_jobs_summary_is_for_mondays_and_the_review_for_sundays() {
        let db = Db::in_memory().await;
        set_enabled(&db, BriefKind::Jobs, true).await.expect("jobs");
        set_enabled(&db, BriefKind::Review, true)
            .await
            .expect("review");

        assert_eq!(
            due(&db, MONDAY_NOON).await.expect("monday"),
            [BriefKind::Jobs]
        );
        let sunday = MONDAY_NOON - 86_400_000;
        assert_eq!(due(&db, sunday).await.expect("sunday"), [BriefKind::Review]);
        let tuesday = MONDAY_NOON + 86_400_000;
        assert!(due(&db, tuesday).await.expect("tuesday").is_empty());
    }

    #[tokio::test]
    async fn the_facts_come_from_the_app() {
        let db = Db::in_memory().await;
        let text = facts(&db, BriefKind::Morning, MONDAY_NOON)
            .await
            .expect("facts");
        assert!(text.contains("Today is 2026-11-16"));
        let review = facts(&db, BriefKind::Review, MONDAY_NOON)
            .await
            .expect("review");
        assert!(review.contains("Nothing finished yet."));
        assert!(prompt(BriefKind::Morning, "x").contains("Facts from the app:\nx"));
    }

    #[tokio::test]
    async fn a_missing_vault_is_refused_before_claude_is_started() {
        let db = Db::in_memory().await;
        vault::set_path(&db, "Z:/no/such/vault")
            .await
            .expect("path");
        let result = run(&db, &std::env::temp_dir(), BriefKind::Morning, MONDAY_NOON).await;
        assert!(result.is_err());
    }
}
