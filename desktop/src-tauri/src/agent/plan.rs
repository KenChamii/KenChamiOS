use super::process::{self, Captured};
use crate::domain::agent_plan::{PlanProposal, SCHEMA};
use crate::error::{AppError, AppResult};
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(300);
// the plan goes in on stdin, but a huge doc would still cost a lot of the user's plan
const MAX_PLAN_CHARS: usize = 100_000;

enum Answer {
    Plan(PlanProposal),
    /// the model answered, but not with a usable plan; worth one more try
    Unusable(String),
}

/// Asks Claude Code to split the plan into phases and tasks. Blocking; run it off the UI thread.
/// An unusable answer is sent back once with the reason, a second one is an error.
pub fn breakdown(cwd: &Path, project_name: &str, plan_md: &str) -> AppResult<PlanProposal> {
    if plan_md.chars().count() > MAX_PLAN_CHARS {
        return Err(AppError::Invalid("the plan is too long to break down"));
    }
    let base = prompt(project_name, plan_md);
    let mut text = base.clone();
    let mut rejected = String::new();
    for _ in 0..2 {
        let captured = process::run(command(cwd), &text, TIMEOUT)?;
        match read_answer(&captured)? {
            Answer::Plan(plan) => return Ok(plan),
            Answer::Unusable(reason) => {
                text = format!(
                    "{base}\n\nYour previous answer was rejected: {reason}\nAnswer again with a valid plan."
                );
                rejected = reason;
            }
        }
    }
    Err(AppError::Agent(format!(
        "Claude did not return a usable plan: {rejected}"
    )))
}

fn command(cwd: &Path) -> std::process::Command {
    let mut cmd = process::command("claude", cwd);
    // no tools, no mcp, no settings files, nothing to ask permission for: it only answers
    cmd.args(["-p", "--output-format", "json", "--json-schema", SCHEMA])
        .args([
            "--restricted",
            "--tools",
            "",
            "--permission-prompts",
            "none",
        ])
        .args(["--strict-mcp-config", "--disable-slash-commands"])
        .arg("--no-session-persistence");
    cmd
}

fn prompt(project_name: &str, plan_md: &str) -> String {
    format!(
        "You turn a project plan into small, buildable tasks.\n\n\
         Project: {project_name}\n\n\
         Rules:\n\
         - Group the work into phases, in build order. Each phase ends with something usable.\n\
         - A phase title is a plain name. Do not start it with \"Phase\" and a number.\n\
         - Each task fits in one sitting (about an hour). Its title is short and starts with a verb.\n\
         - body: what to build and which parts it touches, in one to three sentences.\n\
         - acceptance: how to check the task is done, one or two lines.\n\
         - Use only what the plan says. Do not invent features.\n\n\
         Plan:\n{plan_md}\n"
    )
}

fn read_answer(captured: &Captured) -> AppResult<Answer> {
    let reply: Value = serde_json::from_str(captured.stdout.trim()).map_err(|_| {
        AppError::Agent(format!(
            "Claude Code gave no readable answer{}",
            last_lines(&captured.stderr)
        ))
    })?;
    if reply["is_error"].as_bool().unwrap_or(!captured.success) {
        let text = reply["result"].as_str().unwrap_or("it reported an error");
        return Err(AppError::Agent(format!("Claude Code: {text}")));
    }
    let Some(output) = reply.get("structured_output").filter(|v| !v.is_null()) else {
        return Ok(Answer::Unusable("there was no structured output".into()));
    };
    let plan = serde_json::from_value::<PlanProposal>(output.clone())
        .map_err(|e| e.to_string())
        .and_then(|plan| plan.clean().map_err(|e| e.to_string()));
    Ok(match plan {
        Ok(plan) => Answer::Plan(plan),
        Err(reason) => Answer::Unusable(reason),
    })
}

fn last_lines(stderr: &str) -> String {
    let tail: Vec<_> = stderr.lines().rev().take(3).collect();
    if tail.is_empty() {
        String::new()
    } else {
        format!(": {}", tail.into_iter().rev().collect::<Vec<_>>().join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn captured(stdout: &str) -> Captured {
        Captured {
            stdout: stdout.into(),
            stderr: String::new(),
            success: true,
            code: Some(0),
        }
    }

    fn reply(structured: &str) -> String {
        format!(
            r#"{{"type":"result","is_error":false,"result":"","structured_output":{structured}}}"#
        )
    }

    #[test]
    fn a_valid_answer_becomes_a_plan() {
        let json = reply(
            r#"{"phases":[{"title":" Setup ","tasks":[{"title":"Add table","body":"b","acceptance":"a"}]}]}"#,
        );
        let Ok(Answer::Plan(plan)) = read_answer(&captured(&json)) else {
            panic!("expected a plan");
        };
        assert_eq!(plan.phases[0].title, "Setup");
    }

    #[test]
    fn a_real_cli_reply_becomes_a_plan() {
        let real = include_str!("fixtures/plan_reply.json");
        let Ok(Answer::Plan(plan)) = read_answer(&captured(real)) else {
            panic!("expected a plan");
        };
        assert_eq!(plan.phases.len(), 2);
        assert_eq!(plan.phases[0].tasks.len(), 3);
        assert!(!plan.phases[1].tasks[1].acceptance.is_empty());
    }

    #[test]
    fn a_missing_structured_output_is_unusable() {
        let json = r#"{"type":"result","is_error":false,"result":"sure!"}"#;
        assert!(matches!(
            read_answer(&captured(json)),
            Ok(Answer::Unusable(_))
        ));
    }

    #[test]
    fn a_plan_with_a_blank_title_is_unusable_with_the_reason() {
        let json =
            reply(r#"{"phases":[{"title":"","tasks":[{"title":"t","body":"","acceptance":""}]}]}"#);
        let Ok(Answer::Unusable(reason)) = read_answer(&captured(&json)) else {
            panic!("expected unusable");
        };
        assert!(reason.contains("phase 1"));
    }

    #[test]
    fn extra_fields_are_unusable() {
        let json = reply(r#"{"phases":[],"note":"hi"}"#);
        assert!(matches!(
            read_answer(&captured(&json)),
            Ok(Answer::Unusable(_))
        ));
    }

    #[test]
    fn a_cli_error_is_an_error_not_a_retry() {
        let json = r#"{"type":"result","is_error":true,"result":"Not logged in"}"#;
        let Err(e) = read_answer(&captured(json)) else {
            panic!("expected an error");
        };
        assert!(e.to_string().contains("Not logged in"));
    }

    #[test]
    fn output_that_is_not_json_is_an_error() {
        let out = Captured {
            stdout: "boom".into(),
            stderr: "line one\nline two".into(),
            success: false,
            code: Some(1),
        };
        let Err(e) = read_answer(&out) else {
            panic!("expected an error");
        };
        assert!(e.to_string().contains("line two"));
    }

    #[test]
    fn the_prompt_carries_the_plan_and_project() {
        let text = prompt("Demo", "## Plan\nbuild it");
        assert!(text.contains("Project: Demo") && text.contains("build it"));
    }
}
