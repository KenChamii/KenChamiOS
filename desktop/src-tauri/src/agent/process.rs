use crate::error::{AppError, AppResult};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
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
pub fn run(mut cmd: Command, input: &str, timeout: Duration) -> AppResult<Captured> {
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
    let out = thread::spawn(move || drain(stdout));
    let err = thread::spawn(move || drain(stderr));

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if Instant::now() >= deadline {
            kill_tree(&mut child);
            let _ = child.wait();
            break None;
        }
        thread::sleep(POLL);
    };

    let _ = writer.join();
    let stdout = out.join().unwrap_or_default();
    let stderr = err.join().unwrap_or_default();
    match status {
        Some(status) => Ok(Captured {
            stdout,
            stderr,
            success: status.success(),
        }),
        None => Err(AppError::Agent(format!(
            "Claude Code took longer than {} seconds and was stopped",
            timeout.as_secs()
        ))),
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

    fn shell(script: &str) -> Command {
        let mut cmd = command(
            if cfg!(windows) { "cmd" } else { "sh" },
            &std::env::temp_dir(),
        );
        cmd.args([if cfg!(windows) { "/C" } else { "-c" }, script]);
        cmd
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
        let script = if cfg!(windows) {
            "ping -n 30 127.0.0.1 > nul"
        } else {
            "sleep 30"
        };
        let started = Instant::now();
        let err = run(shell(script), "", Duration::from_millis(400)).expect_err("timeout");
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(err.to_string().contains("was stopped"));
    }

    #[test]
    fn a_missing_program_names_the_cli() {
        let cmd = command("kenchamios-no-such-program", &std::env::temp_dir());
        let err = run(cmd, "", Duration::from_secs(5)).expect_err("missing");
        assert!(err.to_string().contains("not found"));
    }
}
