use crate::error::{AppError, AppResult};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// what the CLI needs to find its login and Git Bash; no api key or token variables
const KEPT_ENV: [&str; 26] = [
    "PATH",
    "PATHEXT",
    "SystemRoot",
    "SystemDrive",
    "windir",
    "ComSpec",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "HOME",
    "USERNAME",
    "APPDATA",
    "LOCALAPPDATA",
    "ProgramData",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramW6432",
    "CommonProgramFiles",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
    "OS",
    "LANG",
    "CLAUDE_CONFIG_DIR",
    "CLAUDE_CODE_GIT_BASH_PATH",
];

const POLL: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct Captured {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    Finished,
    TimedOut,
    /// the stop flag was raised
    Stopped,
    /// `on_line` asked to stop, for example at the turn limit
    Cancelled,
}

#[derive(Debug)]
pub struct Streamed {
    pub ended: Ended,
    /// the process exited with status 0; false for every other ending
    pub success: bool,
    pub code: Option<i32>,
    pub stderr: String,
}

/// A command with an empty environment plus the allowlist, no console window, run in `cwd`.
pub fn command(program: &str, cwd: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear().current_dir(cwd);
    for name in KEPT_ENV {
        if let Some(value) = std::env::var_os(name) {
            cmd.env(name, value);
        }
    }
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Runs `cmd` with `input` on stdin and waits for it, killing the whole process tree on timeout.
pub fn run(cmd: Command, input: &str, timeout: Duration) -> AppResult<Captured> {
    let mut stdout = String::new();
    let never = AtomicBool::new(false);
    let streamed = run_streaming(cmd, input, timeout, &never, |line| {
        stdout.push_str(line);
        stdout.push('\n');
        true
    })?;
    match streamed.ended {
        Ended::Finished => Ok(Captured {
            stdout,
            stderr: streamed.stderr,
            success: streamed.success,
            code: streamed.code,
        }),
        _ => Err(AppError::Agent(format!(
            "Claude Code took longer than {} seconds and was stopped",
            timeout.as_secs()
        ))),
    }
}

/// Like `run`, but hands each stdout line to `on_line` as it arrives. `on_line` returns false to
/// kill the process. The process tree is also killed on timeout and when `stop` is raised.
pub fn run_streaming(
    mut cmd: Command,
    input: &str,
    timeout: Duration,
    stop: &AtomicBool,
    mut on_line: impl FnMut(&str) -> bool,
) -> AppResult<Streamed> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(spawn_error)?;

    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let input = input.to_owned();
    // feed and drain on their own threads, so a full pipe can never stall the child
    let writer = thread::spawn(move || {
        if let Some(mut stdin) = stdin {
            let _ = stdin.write_all(input.as_bytes());
        }
    });
    let (lines, incoming) = mpsc::channel();
    let reader = thread::spawn(move || read_lines(stdout, &lines));
    let err = thread::spawn(move || drain(stderr));

    let deadline = Instant::now() + timeout;
    let mut stdout_closed = false;
    let mut exit = None;
    let ended = loop {
        if stdout_closed {
            thread::sleep(POLL);
            if let Some(status) = child.try_wait()? {
                exit = Some(status);
                break Ended::Finished;
            }
        } else {
            match incoming.recv_timeout(POLL) {
                Ok(line) if !on_line(&line) => break Ended::Cancelled,
                Ok(_) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => stdout_closed = true,
            }
        }
        if stop.load(Ordering::SeqCst) {
            break Ended::Stopped;
        }
        if Instant::now() >= deadline {
            break Ended::TimedOut;
        }
    };
    if exit.is_none() {
        kill_tree(&mut child);
        let _ = child.wait();
    }

    let _ = writer.join();
    let _ = reader.join();
    Ok(Streamed {
        ended,
        success: exit.is_some_and(|status| status.success()),
        code: exit.and_then(|status| status.code()),
        stderr: err.join().unwrap_or_default(),
    })
}

fn read_lines(pipe: Option<impl Read>, lines: &mpsc::Sender<String>) {
    let Some(pipe) = pipe else { return };
    let mut pipe = BufReader::new(pipe);
    let mut raw = Vec::new();
    while matches!(pipe.read_until(b'\n', &mut raw), Ok(n) if n > 0) {
        let line = String::from_utf8_lossy(&raw).trim_end().to_string();
        raw.clear();
        if lines.send(line).is_err() {
            return;
        }
    }
}

fn drain(pipe: Option<impl Read>) -> String {
    let mut bytes = Vec::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut bytes);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn spawn_error(e: std::io::Error) -> AppError {
    if e.kind() == ErrorKind::NotFound {
        AppError::Agent("Claude Code CLI not found. Install it and log in first".into())
    } else {
        AppError::Io(e)
    }
}

// the agent's shells and test processes live below the CLI, so ending the CLI alone is not enough
fn kill_tree(child: &mut Child) {
    if cfg!(windows) {
        let mut taskkill = Command::new("taskkill");
        taskkill
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        taskkill.creation_flags(CREATE_NO_WINDOW);
        let _ = taskkill.status();
    }
    let _ = child.kill();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn shell(script: &str) -> Command {
        let mut cmd = command(
            if cfg!(windows) { "cmd" } else { "sh" },
            &std::env::temp_dir(),
        );
        cmd.args([if cfg!(windows) { "/C" } else { "-c" }, script]);
        cmd
    }

    fn sleeper() -> &'static str {
        if cfg!(windows) {
            "echo first& ping -n 30 127.0.0.1 > nul"
        } else {
            "echo first; sleep 30"
        }
    }

    #[test]
    fn only_allowlisted_variables_reach_the_child() {
        let cmd = command("claude", &std::env::temp_dir());
        let names: Vec<_> = cmd
            .get_envs()
            .map(|(name, _)| name.to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().all(|name| KEPT_ENV.contains(&name.as_str())));
        assert!(!names.iter().any(|name| name.contains("ANTHROPIC")));
    }

    #[test]
    fn captures_output_and_feeds_stdin() {
        let script = if cfg!(windows) { "findstr ." } else { "cat" };
        let out = run(shell(script), "hello\n", Duration::from_secs(20)).expect("run");
        assert!(out.success);
        assert_eq!(out.stdout.trim(), "hello");
    }

    #[test]
    fn a_failing_command_is_reported_not_raised() {
        let out = run(shell("exit 3"), "", Duration::from_secs(20)).expect("run");
        assert!(!out.success);
    }

    #[test]
    fn a_slow_command_is_stopped_at_the_timeout() {
        let started = Instant::now();
        let err = run(shell(sleeper()), "", Duration::from_millis(400)).expect_err("timeout");
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(err.to_string().contains("was stopped"));
    }

    #[test]
    fn a_missing_program_names_the_cli() {
        let cmd = command("kenchamios-no-such-program", &std::env::temp_dir());
        let err = run(cmd, "", Duration::from_secs(5)).expect_err("missing");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn lines_arrive_one_by_one_and_the_exit_is_reported() {
        let script = if cfg!(windows) {
            "echo one& echo two"
        } else {
            "echo one; echo two"
        };
        let mut seen = Vec::new();
        let never = AtomicBool::new(false);
        let streamed = run_streaming(shell(script), "", Duration::from_secs(20), &never, |l| {
            seen.push(l.to_string());
            true
        })
        .expect("run");

        assert_eq!(streamed.ended, Ended::Finished);
        assert!(streamed.success);
        assert_eq!(seen, ["one", "two"]);
    }

    #[test]
    fn on_line_can_cancel_the_run() {
        let never = AtomicBool::new(false);
        let started = Instant::now();
        let streamed = run_streaming(
            shell(sleeper()),
            "",
            Duration::from_secs(60),
            &never,
            |_| false,
        )
        .expect("run");

        assert_eq!(streamed.ended, Ended::Cancelled);
        assert!(!streamed.success);
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn the_stop_flag_ends_a_running_process() {
        let stop = Arc::new(AtomicBool::new(false));
        let raiser = stop.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(300));
            raiser.store(true, Ordering::SeqCst);
        });
        let started = Instant::now();
        let streamed = run_streaming(shell(sleeper()), "", Duration::from_secs(60), &stop, |_| {
            true
        })
        .expect("run");

        assert_eq!(streamed.ended, Ended::Stopped);
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_timeout_is_reported_as_timed_out() {
        let never = AtomicBool::new(false);
        let streamed = run_streaming(
            shell(sleeper()),
            "",
            Duration::from_millis(400),
            &never,
            |_| true,
        )
        .expect("run");
        assert_eq!(streamed.ended, Ended::TimedOut);
    }
}
