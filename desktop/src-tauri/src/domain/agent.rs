use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Review,
    Done,
    Failed,
    Discarded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunAction {
    Start,
    Finish,
    Fail,
    Requeue,
    Merge,
    Discard,
}

/// The status after `action`, or an error when the run cannot do that now.
pub fn transition(from: RunStatus, action: RunAction) -> AppResult<RunStatus> {
    use RunAction as A;
    use RunStatus as S;
    match (from, action) {
        (S::Queued, A::Start) => Ok(S::Running),
        (S::Running, A::Finish) => Ok(S::Review),
        (S::Running, A::Fail) => Ok(S::Failed),
        (S::Review | S::Failed, A::Requeue) => Ok(S::Queued),
        (S::Review, A::Merge) => Ok(S::Done),
        (S::Queued | S::Review | S::Failed, A::Discard) => Ok(S::Discarded),
        _ => Err(AppError::Invalid(
            "the run cannot do that in its current state",
        )),
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AgentRun {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub project_id: String,
    pub task_id: String,
    pub status: RunStatus,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
    pub base_branch: Option<String>,
    pub base_commit: Option<String>,
    pub commit_sha: Option<String>,
    pub feedback: String,
    pub error: Option<String>,
    pub turns: i64,
    pub test_exit_code: Option<i64>,
    pub log_path: String,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

/// A run with the title of its task, for the board.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AgentRunView {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub run: AgentRun,
    pub task_title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    #[serde(rename = "dontAsk")]
    DontAsk,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
}

impl PermissionMode {
    pub fn as_flag(self) -> &'static str {
        match self {
            Self::DontAsk => "dontAsk",
            Self::AcceptEdits => "acceptEdits",
        }
    }
}

/// Built-in CLI tools a project may allow; Bash is only ever allowed for the test command.
pub const KNOWN_TOOLS: [&str; 6] = ["Read", "Edit", "Write", "Glob", "Grep", "Bash"];
const MAX_TIMEOUT_MIN: u32 = 240;
const MAX_TURNS: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSettings {
    /// empty means the default rules
    pub rules: String,
    pub test_command: String,
    pub tools: Vec<String>,
    pub permission_mode: PermissionMode,
    pub timeout_min: u32,
    pub max_turns: u32,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            rules: String::new(),
            test_command: String::new(),
            tools: KNOWN_TOOLS.iter().map(|t| t.to_string()).collect(),
            permission_mode: PermissionMode::DontAsk,
            timeout_min: 30,
            max_turns: 40,
        }
    }
}

impl AgentSettings {
    /// Trimmed, deduplicated and checked, in the order of `KNOWN_TOOLS`.
    pub fn clean(self) -> AppResult<Self> {
        if let Some(unknown) = self
            .tools
            .iter()
            .find(|t| !KNOWN_TOOLS.contains(&t.as_str()))
        {
            return Err(AppError::Agent(format!("unknown agent tool: {unknown}")));
        }
        if !(1..=MAX_TIMEOUT_MIN).contains(&self.timeout_min) {
            return Err(AppError::Invalid("the timeout must be 1 to 240 minutes"));
        }
        if !(1..=MAX_TURNS).contains(&self.max_turns) {
            return Err(AppError::Invalid("the turn limit must be 1 to 200"));
        }
        let test_command = self.test_command.trim().to_string();
        if test_command.contains(['\n', '\r']) {
            return Err(AppError::Invalid("the test command must be one line"));
        }
        Ok(Self {
            rules: self.rules.trim().to_string(),
            tools: KNOWN_TOOLS
                .iter()
                .filter(|known| self.tools.iter().any(|t| t == *known))
                .map(|t| t.to_string())
                .collect(),
            test_command,
            ..self
        })
    }

    /// `--tools` value: the allowed built-in tools; Bash only when there is a test command.
    pub fn cli_tools(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|t| *t != "Bash" || !self.test_command.is_empty())
            .cloned()
            .collect()
    }

    /// `--allowedTools` value: the same tools, Bash narrowed to the test command.
    pub fn cli_allowed(&self) -> Vec<String> {
        self.cli_tools()
            .into_iter()
            .flat_map(|tool| {
                if tool == "Bash" {
                    vec![
                        format!("Bash({})", self.test_command),
                        format!("Bash({} *)", self.test_command),
                    ]
                } else {
                    vec![tool]
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RunAction as A;
    use RunStatus as S;

    const ALL: [RunStatus; 6] = [
        S::Queued,
        S::Running,
        S::Review,
        S::Done,
        S::Failed,
        S::Discarded,
    ];
    const ACTIONS: [RunAction; 6] = [
        A::Start,
        A::Finish,
        A::Fail,
        A::Requeue,
        A::Merge,
        A::Discard,
    ];

    #[test]
    fn happy_path_runs_from_queue_to_done() {
        let mut status = S::Queued;
        for action in [
            A::Start,
            A::Finish,
            A::Requeue,
            A::Start,
            A::Finish,
            A::Merge,
        ] {
            status = transition(status, action).expect("allowed");
        }
        assert_eq!(status, S::Done);
    }

    #[test]
    fn only_the_listed_moves_are_allowed() {
        let allowed = [
            (S::Queued, A::Start, S::Running),
            (S::Running, A::Finish, S::Review),
            (S::Running, A::Fail, S::Failed),
            (S::Review, A::Requeue, S::Queued),
            (S::Failed, A::Requeue, S::Queued),
            (S::Review, A::Merge, S::Done),
            (S::Queued, A::Discard, S::Discarded),
            (S::Review, A::Discard, S::Discarded),
            (S::Failed, A::Discard, S::Discarded),
        ];
        for from in ALL {
            for action in ACTIONS {
                let expected = allowed
                    .iter()
                    .find(|(f, a, _)| *f == from && *a == action)
                    .map(|(_, _, to)| *to);
                assert_eq!(
                    transition(from, action).ok(),
                    expected,
                    "{from:?} {action:?}"
                );
            }
        }
    }

    #[test]
    fn done_and_discarded_are_final() {
        for action in ACTIONS {
            assert!(transition(S::Done, action).is_err());
            assert!(transition(S::Discarded, action).is_err());
        }
    }

    #[test]
    fn clean_trims_orders_and_checks_limits() {
        let settings = AgentSettings {
            rules: "  keep it small \n".into(),
            test_command: " npm test ".into(),
            tools: vec!["Bash".into(), "Read".into(), "Read".into()],
            ..AgentSettings::default()
        }
        .clean()
        .expect("clean");
        assert_eq!(settings.rules, "keep it small");
        assert_eq!(settings.test_command, "npm test");
        assert_eq!(settings.tools, ["Read", "Bash"]);

        let bad = |change: fn(&mut AgentSettings)| {
            let mut settings = AgentSettings::default();
            change(&mut settings);
            settings.clean().is_err()
        };
        assert!(bad(|s| s.tools = vec!["WebFetch".into()]));
        assert!(bad(|s| s.timeout_min = 0));
        assert!(bad(|s| s.max_turns = 201));
        assert!(bad(|s| s.test_command = "npm test\nrm -rf /".into()));
    }

    #[test]
    fn bash_is_only_allowed_for_the_test_command() {
        let without = AgentSettings::default();
        assert_eq!(
            without.cli_tools(),
            ["Read", "Edit", "Write", "Glob", "Grep"]
        );

        let with = AgentSettings {
            test_command: "cargo test".into(),
            ..AgentSettings::default()
        };
        assert_eq!(with.cli_tools().last().map(String::as_str), Some("Bash"));
        assert_eq!(
            with.cli_allowed()[5..],
            ["Bash(cargo test)", "Bash(cargo test *)"]
        );
    }

    #[test]
    fn permission_modes_use_the_cli_names() {
        let json = serde_json::to_string(&PermissionMode::DontAsk).expect("json");
        assert_eq!(json, "\"dontAsk\"");
        assert_eq!(PermissionMode::AcceptEdits.as_flag(), "acceptEdits");
        assert!(serde_json::from_str::<PermissionMode>("\"bypassPermissions\"").is_err());
    }
}
