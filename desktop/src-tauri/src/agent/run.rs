use super::process::{self, Ended};
use super::prompt::{self, PromptParts};
use super::stream::{self, StreamEvent, TurnCounter};
use super::{review, worktree};
use crate::db::{agent_runs, projects, tasks, templates};
use crate::domain::agent::{AgentRun, AgentSettings, RunAction};
use crate::domain::render::render;
use crate::domain::{TaskStatus, TemplateKind};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::workflow::context::task_context;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const WORKTREES_DIR: &str = "worktrees";
const LOGS_DIR: &str = "agent-logs";
const TEST_TIMEOUT: Duration = Duration::from_secs(600);
const MAX_ERROR_LINES: usize = 5;

pub fn log_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(LOGS_DIR)
}

pub fn test_output_path(log_path: &str) -> PathBuf {
    Path::new(log_path).with_extension("test.txt")
}

/// Which run is going, so the board can stop it. One run at a time.
#[derive(Default)]
pub struct Runner {
    busy: AtomicBool,
    current: Mutex<Option<(String, Arc<AtomicBool>)>>,
}

impl Runner {
    fn claim(&self) -> bool {
        self.busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    fn release(&self) {
        self.set_current(None);
        self.busy.store(false, Ordering::SeqCst);
    }

    fn set_current(&self, run: Option<(String, Arc<AtomicBool>)>) {
        if let Ok(mut current) = self.current.lock() {
            *current = run;
        }
    }

    /// Raises the stop flag of the run when it is the one going.
    pub fn stop(&self, run_id: &str) -> bool {
        let Ok(current) = self.current.lock() else {
            return false;
        };
        match current.as_ref() {
            Some((id, flag)) if id == run_id => {
                flag.store(true, Ordering::SeqCst);
                true
            }
            _ => false,
        }
    }
}

/// Starts the queued runs, one after the other, in the background.
pub fn kick(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { drain(&app).await });
}

async fn drain(app: &AppHandle) {
    let state = app.state::<AppState>();
    loop {
        if !state.agent.claim() {
            return;
        }
        let next = agent_runs::next_queued(&state.db).await;
        let Ok(Some(run)) = next else {
            state.agent.release();
            // a run queued while we were letting go would otherwise wait for the next kick
            if matches!(agent_runs::next_queued(&state.db).await, Ok(Some(_))) {
                continue;
            }
            return;
        };
        let stop = Arc::new(AtomicBool::new(false));
        state
            .agent
            .set_current(Some((run.id.clone(), stop.clone())));
        execute(app, &run, &stop).await;
        state.agent.release();
    }
}

async fn execute(app: &AppHandle, run: &AgentRun, stop: &Arc<AtomicBool>) {
    let state = app.state::<AppState>();
    if let Err(error) = run_agent(app, run, stop).await {
        let message = error.to_string();
        let _ = agent_runs::apply(&state.db, &run.id, RunAction::Fail, Some(&message)).await;
        let _ = tasks::move_to(&state.db, &run.task_id, TaskStatus::Todo, None, None).await;
    }
    let _ = app.emit("agent-run-changed", &run.id);
}

async fn run_agent(app: &AppHandle, run: &AgentRun, stop: &Arc<AtomicBool>) -> AppResult<()> {
    let state = app.state::<AppState>();
    let db = &state.db;
    agent_runs::apply(db, &run.id, RunAction::Start, None).await?;
    tasks::move_to(db, &run.task_id, TaskStatus::Doing, None, None).await?;
    let _ = app.emit("agent-run-changed", &run.id);

    let task = tasks::get(db, &run.task_id).await?;
    let project = projects::get(db, &run.project_id).await?.project;
    let repo_path = project
        .repo_path
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| AppError::Agent("link a Git repo to the project first".into()))?;
    let settings = agent_runs::get_settings(db, &run.project_id).await?;

    let (branch, cwd) = match (&run.branch, &run.worktree_path) {
        (Some(branch), Some(path)) if Path::new(path).is_dir() => {
            (branch.clone(), PathBuf::from(path))
        }
        _ => {
            let made = worktree::create(
                &repo_path,
                &state.data_dir.join(WORKTREES_DIR),
                &run.id,
                &task.title,
            )?;
            agent_runs::set_worktree(db, &run.id, &made).await?;
            (made.branch, made.path)
        }
    };

    let ctx = task_context(db, &run.task_id).await?;
    let brief = render(
        &templates::get_default(db, TemplateKind::Brief).await?.body,
        &ctx,
    )?;
    let task_prompt = render(
        &templates::get_default(db, TemplateKind::TaskPrompt)
            .await?
            .body,
        &ctx,
    )?;
    let text = prompt::build(
        &settings,
        &PromptParts {
            brief: &brief,
            task_prompt: &task_prompt,
            branch: &branch,
            feedback: &run.feedback,
        },
    )?;

    let job = AgentJob {
        log_path: PathBuf::from(&run.log_path),
        cwd: cwd.clone(),
        settings: settings.clone(),
        prompt: text,
        stop: stop.clone(),
    };
    let outcome = tauri::async_runtime::spawn_blocking(move || stream_agent(&job))
        .await
        .map_err(|e| AppError::Agent(e.to_string()))??;
    if let Some(reason) = outcome.failure() {
        agent_runs::set_progress(db, &run.id, outcome.turns, None).await?;
        return Err(AppError::Agent(reason));
    }

    let exit = run_tests(&settings, &cwd, &run.log_path).await;
    agent_runs::set_progress(db, &run.id, outcome.turns, exit).await?;
    agent_runs::apply(db, &run.id, RunAction::Finish, None).await?;
    tasks::move_to(db, &run.task_id, TaskStatus::Review, None, None).await?;
    Ok(())
}

struct AgentJob {
    log_path: PathBuf,
    cwd: PathBuf,
    settings: AgentSettings,
    prompt: String,
    stop: Arc<AtomicBool>,
}

struct Outcome {
    ended: Ended,
    success: bool,
    turns: i64,
    over_limit: bool,
    limit: u32,
    timeout_min: u32,
    /// the final `result` event: (is_error, text)
    result: Option<(bool, String)>,
    stderr: String,
}

impl Outcome {
    /// Why the run did not finish well, or `None` when it did.
    fn failure(&self) -> Option<String> {
        if self.over_limit {
            return Some(format!("turn limit reached ({})", self.limit));
        }
        match self.ended {
            Ended::Stopped => return Some("stopped".into()),
            Ended::TimedOut => {
                return Some(format!("timed out after {} minutes", self.timeout_min));
            }
            Ended::Cancelled => return Some("stopped".into()),
            Ended::Finished => {}
        }
        match &self.result {
            Some((true, text)) => Some(format!("Claude Code: {text}")),
            Some((false, _)) => None,
            None if self.success => Some("Claude Code ended without a result".into()),
            None => Some(format!(
                "Claude Code stopped without a result{}",
                last_lines(&self.stderr)
            )),
        }
    }
}

fn last_lines(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(MAX_ERROR_LINES);
    if lines.is_empty() {
        String::new()
    } else {
        format!(": {}", lines[start..].join(" | "))
    }
}

fn open_log(path: &Path) -> AppResult<File> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

// blocking: runs on its own thread, feeds the log file and the live view line by line
fn stream_agent(job: &AgentJob) -> AppResult<Outcome> {
    let settings = &job.settings;
    let mut cmd = process::command("claude", &job.cwd);
    cmd.args([
        "-p",
        "--output-format",
        "stream-json",
        "--verbose",
        "--restricted",
    ])
    .args(["--tools", &settings.cli_tools().join(",")])
    .arg("--allowedTools")
    .args(settings.cli_allowed())
    .args(["--permission-mode", settings.permission_mode.as_flag()])
    .args(["--permission-prompts", "none"])
    .args(["--strict-mcp-config", "--disable-slash-commands"])
    .arg("--no-session-persistence");

    let mut log = open_log(&job.log_path)?;
    let mut counter = TurnCounter::default();
    let mut turns = 0usize;
    let mut over_limit = false;
    let mut result = None;
    let limit = settings.max_turns as usize;
    let streamed = process::run_streaming(
        cmd,
        &job.prompt,
        Duration::from_secs(u64::from(settings.timeout_min) * 60),
        &job.stop,
        |line| {
            let _ = writeln!(log, "{line}");
            for event in stream::parse_line(line) {
                turns = counter.observe(&event);
                if let StreamEvent::Result { is_error, text, .. } = &event {
                    result = Some((*is_error, text.clone()));
                }
            }
            over_limit = turns > limit;
            !over_limit
        },
    )?;
    Ok(Outcome {
        ended: streamed.ended,
        success: streamed.success,
        turns: turns as i64,
        over_limit,
        limit: settings.max_turns,
        timeout_min: settings.timeout_min,
        result,
        stderr: streamed.stderr,
    })
}

/// Runs the project's test command in the worktree. `None` when there is none, -1 when it
/// could not run or timed out; the output goes next to the log.
async fn run_tests(settings: &AgentSettings, cwd: &Path, log_path: &str) -> Option<i64> {
    if settings.test_command.is_empty() {
        return None;
    }
    let command = settings.test_command.clone();
    let cwd = cwd.to_path_buf();
    let out_path = test_output_path(log_path);
    let ran = tauri::async_runtime::spawn_blocking(move || {
        let mut cmd = process::command(if cfg!(windows) { "cmd" } else { "sh" }, &cwd);
        cmd.args([if cfg!(windows) { "/C" } else { "-c" }, &command]);
        let (text, code) = match process::run(cmd, "", TEST_TIMEOUT) {
            Ok(done) => (
                format!("{}{}", done.stdout, done.stderr),
                done.code.map_or(-1, i64::from),
            ),
            Err(error) => (error.to_string(), -1),
        };
        let _ = fs::write(out_path, text);
        code
    })
    .await;
    Some(ran.unwrap_or(-1))
}

/// Diff of a run's worktree against where it started.
pub fn diff_of(run: &AgentRun) -> AppResult<review::RunDiff> {
    match (&run.worktree_path, &run.base_commit) {
        (Some(path), Some(base)) => review::diff(Path::new(path), base),
        _ => Err(AppError::Invalid("the run has no worktree yet")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(ended: Ended) -> Outcome {
        Outcome {
            ended,
            success: true,
            turns: 3,
            over_limit: false,
            limit: 40,
            timeout_min: 30,
            result: Some((false, "done".into())),
            stderr: String::new(),
        }
    }

    #[test]
    fn a_clean_finish_has_no_failure() {
        assert_eq!(outcome(Ended::Finished).failure(), None);
    }

    #[test]
    fn stop_timeout_and_turn_limit_each_have_a_reason() {
        assert_eq!(
            outcome(Ended::Stopped).failure().as_deref(),
            Some("stopped")
        );
        assert!(
            outcome(Ended::TimedOut)
                .failure()
                .is_some_and(|r| r.contains("30 minutes"))
        );
        let mut over = outcome(Ended::Cancelled);
        over.over_limit = true;
        assert_eq!(over.failure().as_deref(), Some("turn limit reached (40)"));
    }

    #[test]
    fn an_error_result_becomes_the_run_error() {
        let mut failed = outcome(Ended::Finished);
        failed.result = Some((true, "not logged in".into()));
        assert_eq!(
            failed.failure().as_deref(),
            Some("Claude Code: not logged in")
        );
    }

    #[test]
    fn a_crash_without_a_result_shows_the_last_stderr_lines() {
        let mut crashed = outcome(Ended::Finished);
        crashed.success = false;
        crashed.result = None;
        crashed.stderr = "a\n\nb\n".into();
        let reason = crashed.failure().expect("failure");
        assert!(reason.ends_with(": a | b"), "{reason}");
    }

    #[test]
    fn stop_only_reaches_the_run_that_is_going() {
        let runner = Runner::default();
        let flag = Arc::new(AtomicBool::new(false));
        runner.set_current(Some(("run-1".into(), flag.clone())));
        assert!(!runner.stop("run-2"));
        assert!(!flag.load(Ordering::SeqCst));
        assert!(runner.stop("run-1"));
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn only_one_claim_at_a_time() {
        let runner = Runner::default();
        assert!(runner.claim());
        assert!(!runner.claim());
        runner.release();
        assert!(runner.claim());
    }

    #[test]
    fn test_output_sits_next_to_the_log() {
        assert_eq!(
            test_output_path("logs/abc.jsonl"),
            PathBuf::from("logs/abc.test.txt")
        );
    }
}
