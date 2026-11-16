use crate::domain::agent::AgentSettings;
use crate::error::AppResult;
use minijinja::{Environment, context};

const RULES: &str = include_str!("../../templates/agent_rules.md.j2");
const RUN: &str = include_str!("../../templates/agent_run.md.j2");

pub struct PromptParts<'a> {
    pub brief: &'a str,
    pub task_prompt: &'a str,
    pub branch: &'a str,
    pub feedback: &'a str,
}

/// The default rules with the test command, unless the project wrote its own.
pub fn rules(settings: &AgentSettings) -> AppResult<String> {
    if !settings.rules.is_empty() {
        return Ok(settings.rules.clone());
    }
    let text = Environment::new()
        .render_str(RULES, context! { test_command => &settings.test_command })?;
    Ok(text.trim().to_string())
}

/// What goes to the agent on stdin: brief, task, rules, branch and any feedback.
pub fn build(settings: &AgentSettings, parts: &PromptParts) -> AppResult<String> {
    let text = Environment::new().render_str(
        RUN,
        context! {
            brief => parts.brief.trim(),
            task_prompt => parts.task_prompt.trim(),
            rules => rules(settings)?,
            branch => parts.branch,
            feedback => parts.feedback.trim(),
        },
    )?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts<'a>(feedback: &'a str) -> PromptParts<'a> {
        PromptParts {
            brief: "# Demo\nA demo project.",
            task_prompt: "Goal: add a table",
            branch: "agent/add-a-table",
            feedback,
        }
    }

    #[test]
    fn the_default_rules_name_the_test_command() {
        let settings = AgentSettings {
            test_command: "cargo test".into(),
            ..AgentSettings::default()
        };
        let rules = rules(&settings).expect("rules");
        assert!(rules.contains("Run `cargo test`"));
        assert!(rules.contains("Do not use Git"));
    }

    #[test]
    fn without_a_test_command_the_agent_is_told_it_cannot_run_commands() {
        let rules = rules(&AgentSettings::default()).expect("rules");
        assert!(rules.contains("cannot run commands"));
    }

    #[test]
    fn project_rules_replace_the_defaults() {
        let settings = AgentSettings {
            rules: "only touch src/".into(),
            ..AgentSettings::default()
        };
        assert_eq!(rules(&settings).expect("rules"), "only touch src/");
    }

    #[test]
    fn the_prompt_has_brief_task_rules_and_branch() {
        let text = build(&AgentSettings::default(), &parts("")).expect("prompt");
        for expected in [
            "# Demo",
            "Goal: add a table",
            "## Rules",
            "agent/add-a-table",
        ] {
            assert!(text.contains(expected), "{expected}");
        }
        assert!(!text.contains("Feedback"));
    }

    #[test]
    fn feedback_is_added_on_a_rerun() {
        let text = build(&AgentSettings::default(), &parts("use a grid")).expect("prompt");
        assert!(text.contains("## Feedback on your previous attempt"));
        assert!(text.contains("use a grid"));
    }
}
