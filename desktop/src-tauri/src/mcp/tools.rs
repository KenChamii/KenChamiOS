use crate::db::{Db, jobs, planner, projects, tasks, time_entries};
use crate::domain::TaskStatus;
use crate::domain::job::{JobStage, JobView};
use crate::domain::project::{INBOX_ID, ProjectFilter, Stage};
use crate::domain::task::{Priority, TaskCreate};
use crate::domain::time_entry::TimeEntryInput;
use crate::error::{AppError, AppResult};
use crate::vault::dates::{day_start_ms, day_string, days_of, parse_day};
use serde_json::{Value, json};

const MAX_MINUTES: i64 = 24 * 60;

/// What a tool did, and how to reverse it when it changed something.
pub struct Outcome {
    pub text: String,
    pub undo: Option<Value>,
}

fn done(text: impl Into<String>) -> AppResult<Outcome> {
    Ok(Outcome {
        text: text.into(),
        undo: None,
    })
}

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false
        }
    })
}

/// The tools the server lists.
pub fn definitions() -> Value {
    let text = |what: &str| json!({ "type": "string", "description": what });
    let project = text("project name, id or slug");
    Value::Array(vec![
        tool(
            "read_today",
            "What is planned today, the open inbox count and job follow-ups that are due.",
            json!({}),
            &[],
        ),
        tool(
            "list_projects",
            "Projects with their stage and id.",
            json!({}),
            &[],
        ),
        tool(
            "list_tasks",
            "Open tasks of one project.",
            json!({ "project": project }),
            &["project"],
        ),
        tool(
            "list_jobs",
            "Job applications with their stage and id.",
            json!({}),
            &[],
        ),
        tool(
            "add_task",
            "Add a task to a project.",
            json!({
                "project": project,
                "title": text("short title that starts with a verb"),
                "due": text("optional due day, YYYY-MM-DD")
            }),
            &["project", "title"],
        ),
        tool(
            "add_inbox_item",
            "Capture a thought in the Inbox.",
            json!({ "text": text("what to remember") }),
            &["text"],
        ),
        tool(
            "log_time",
            "Log time that was already spent on a project, ending now.",
            json!({
                "project": project,
                "minutes": { "type": "integer", "description": "1 to 1440" },
                "note": text("what was done")
            }),
            &["project", "minutes"],
        ),
        tool(
            "move_project_stage",
            "Move a project to another stage: idea, plan, build, review or done.",
            json!({ "project": project, "stage": text("idea, plan, build, review or done") }),
            &["project", "stage"],
        ),
        tool(
            "move_job_stage",
            "Move a job application: saved, applied, screening, interview, offer, accepted, rejected or withdrawn.",
            json!({
                "job": text("application id, or part of 'company - role'"),
                "stage": text("the new stage")
            }),
            &["job", "stage"],
        ),
        tool(
            "undo_last",
            "Reverse the newest change made through this server that was not undone yet.",
            json!({}),
            &[],
        ),
    ])
}

fn arg<'a>(args: &'a Value, name: &str) -> AppResult<&'a str> {
    args[name]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Agent(format!("the argument '{name}' is required")))
}

fn plain_error(text: String) -> AppError {
    AppError::Agent(text)
}

async fn find_project(db: &Db, text: &str) -> AppResult<(String, String)> {
    let all = projects::list(db, &ProjectFilter::default()).await?;
    let wanted = text.to_lowercase();
    let live: Vec<_> = all
        .iter()
        .map(|s| &s.project)
        .filter(|p| p.id != INBOX_ID)
        .collect();
    if let Some(p) = live
        .iter()
        .find(|p| p.id == text || p.name.to_lowercase() == wanted || p.slug == wanted)
    {
        return Ok((p.id.clone(), p.name.clone()));
    }
    let partial: Vec<_> = live
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&wanted))
        .collect();
    match partial.as_slice() {
        [one] => Ok((one.id.clone(), one.name.clone())),
        [] => Err(plain_error(format!("no project matches '{text}'"))),
        many => Err(plain_error(format!(
            "'{text}' matches several projects: {}",
            many.iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn job_label(view: &JobView) -> String {
    format!("{} - {}", view.job.company, view.job.role)
}

async fn find_job(db: &Db, text: &str, now: i64) -> AppResult<JobView> {
    let all = jobs::list(db, now).await?;
    let wanted = text.to_lowercase();
    if let Some(job) = all.iter().find(|v| v.job.id == text) {
        return Ok(job.clone());
    }
    let partial: Vec<_> = all
        .iter()
        .filter(|v| job_label(v).to_lowercase().contains(&wanted))
        .collect();
    match partial.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(plain_error(format!("no application matches '{text}'"))),
        many => Err(plain_error(format!(
            "'{text}' matches several applications: {}",
            many.iter()
                .map(|v| job_label(v))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn task_input(project_id: &str, title: &str, due_at: Option<i64>) -> TaskCreate {
    TaskCreate {
        project_id: project_id.to_string(),
        parent_id: None,
        title: title.to_string(),
        body_md: String::new(),
        acceptance_md: String::new(),
        related_files: Vec::new(),
        priority: Priority::Medium,
        due_at,
        tags: Vec::new(),
        recurrence: None,
    }
}

/// Runs one tool. Reads change nothing; writes return how to undo them.
pub async fn call(db: &Db, name: &str, args: &Value, now: i64) -> AppResult<Outcome> {
    match name {
        "read_today" => read_today(db, now).await,
        "list_projects" => {
            let lines: Vec<String> = projects::list(db, &ProjectFilter::default())
                .await?
                .into_iter()
                .map(|s| s.project)
                .filter(|p| p.id != INBOX_ID)
                .map(|p| format!("- {} [{}] id {}", p.name, p.stage.as_str(), p.id))
                .collect();
            done(if lines.is_empty() {
                "No projects.".to_string()
            } else {
                lines.join("\n")
            })
        }
        "list_tasks" => {
            let (id, name) = find_project(db, arg(args, "project")?).await?;
            let lines: Vec<String> = tasks::list(db, &id)
                .await?
                .into_iter()
                .filter(|t| t.status != TaskStatus::Done && t.parent_id.is_none())
                .map(|t| format!("- {} ({:?}) id {}", t.title, t.status, t.id))
                .collect();
            done(if lines.is_empty() {
                format!("No open tasks in {name}.")
            } else {
                lines.join("\n")
            })
        }
        "list_jobs" => {
            let lines: Vec<String> = jobs::list(db, now)
                .await?
                .iter()
                .map(|v| {
                    format!(
                        "- {} [{}] id {}",
                        job_label(v),
                        v.job.stage.as_str(),
                        v.job.id
                    )
                })
                .collect();
            done(if lines.is_empty() {
                "No applications.".to_string()
            } else {
                lines.join("\n")
            })
        }
        "add_task" => {
            let (project_id, project_name) = find_project(db, arg(args, "project")?).await?;
            let due_at = match args["due"]
                .as_str()
                .map(str::trim)
                .filter(|d| !d.is_empty())
            {
                Some(day) => {
                    Some(day_start_ms(parse_day(day).ok_or_else(|| {
                        plain_error("the due day must be YYYY-MM-DD".into())
                    })?))
                }
                None => None,
            };
            let task =
                tasks::create(db, task_input(&project_id, arg(args, "title")?, due_at)).await?;
            Ok(Outcome {
                text: format!("Added '{}' to {project_name}.", task.title),
                undo: Some(json!({ "kind": "delete_task", "id": task.id })),
            })
        }
        "add_inbox_item" => {
            let task = tasks::create(db, task_input(INBOX_ID, arg(args, "text")?, None)).await?;
            Ok(Outcome {
                text: format!("Added '{}' to the Inbox.", task.title),
                undo: Some(json!({ "kind": "delete_task", "id": task.id })),
            })
        }
        "log_time" => {
            let (project_id, project_name) = find_project(db, arg(args, "project")?).await?;
            let minutes = args["minutes"]
                .as_i64()
                .filter(|m| (1..=MAX_MINUTES).contains(m))
                .ok_or_else(|| plain_error("minutes must be 1 to 1440".into()))?;
            let entry = time_entries::create_manual(
                db,
                &project_id,
                TimeEntryInput {
                    task_id: None,
                    started_at: now - minutes * 60_000,
                    duration_s: minutes * 60,
                    note: args["note"].as_str().unwrap_or("").to_string(),
                },
            )
            .await?;
            Ok(Outcome {
                text: format!("Logged {minutes} minutes on {project_name}."),
                undo: Some(json!({ "kind": "delete_time", "id": entry.id })),
            })
        }
        "move_project_stage" => {
            let (id, name) = find_project(db, arg(args, "project")?).await?;
            let stage: Stage = serde_json::from_value(json!(arg(args, "stage")?.to_lowercase()))
                .map_err(|_| {
                    plain_error("the stage must be idea, plan, build, review or done".into())
                })?;
            let before = projects::get(db, &id).await?.project.stage;
            projects::set_stage(db, &id, stage).await?;
            Ok(Outcome {
                text: format!("{name} is now in {}.", stage.as_str()),
                undo: Some(json!({ "kind": "project_stage", "id": id, "from": before.as_str() })),
            })
        }
        "move_job_stage" => {
            let view = find_job(db, arg(args, "job")?, now).await?;
            let stage: JobStage = serde_json::from_value(json!(arg(args, "stage")?.to_lowercase()))
                .map_err(|_| plain_error("that is not a job stage".into()))?;
            let today = day_string(days_of(now));
            jobs::move_stage(db, &view.job.id, stage, &today).await?;
            Ok(Outcome {
                text: format!("{} is now {}.", job_label(&view), stage.as_str()),
                undo: Some(json!({
                    "kind": "job_stage",
                    "id": view.job.id,
                    "from": view.job.stage.as_str()
                })),
            })
        }
        other => Err(plain_error(format!("unknown tool: {other}"))),
    }
}

async fn read_today(db: &Db, now: i64) -> AppResult<Outcome> {
    let today = day_string(days_of(now));
    let planned = planner::planned(db, &today, &today).await?;
    let mut lines = vec![format!("Today is {today}.")];
    if planned.is_empty() {
        lines.push("Nothing is planned for today.".into());
    } else {
        lines.push("Planned today:".into());
        for task in &planned {
            let mark = if task.status == TaskStatus::Done {
                "x"
            } else {
                " "
            };
            lines.push(format!("- [{mark}] {} ({})", task.title, task.project_name));
        }
    }
    let inbox = tasks::list(db, INBOX_ID)
        .await?
        .iter()
        .filter(|t| t.status != TaskStatus::Done)
        .count();
    lines.push(format!("Open inbox items: {inbox}."));
    for view in jobs::list(db, now)
        .await?
        .iter()
        .filter(|v| v.due || v.stale)
    {
        let why = if view.due {
            "next step due"
        } else {
            "no news for a week"
        };
        lines.push(format!("Job: {} ({why}).", job_label(view)));
    }
    done(lines.join("\n"))
}

/// Reverses one recorded write.
pub async fn undo(db: &Db, undo: &Value, now: i64) -> AppResult<String> {
    let id = undo["id"].as_str().unwrap_or_default();
    match undo["kind"].as_str().unwrap_or_default() {
        "delete_task" => {
            tasks::delete(db, id).await?;
            Ok("Removed the task (it is in the trash).".into())
        }
        "delete_time" => {
            time_entries::delete(db, id).await?;
            Ok("Removed the time entry.".into())
        }
        "project_stage" => {
            let from: Stage = serde_json::from_value(undo["from"].clone())?;
            projects::set_stage(db, id, from).await?;
            Ok(format!("The project is back in {}.", from.as_str()))
        }
        "job_stage" => {
            let from: JobStage = serde_json::from_value(undo["from"].clone())?;
            let today = day_string(days_of(now));
            jobs::move_stage(db, id, from, &today).await?;
            Ok(format!("The application is back in {}.", from.as_str()))
        }
        _ => Err(plain_error("this change cannot be undone".into())),
    }
}
