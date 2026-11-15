use super::TaskCreate;
use super::task::Priority;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

const MAX_PHASES: usize = 12;
const MAX_TASKS: usize = 20;
const MAX_TITLE: usize = 120;

/// Passed to the CLI so the answer is checked before the app sees it.
pub const SCHEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["phases"],"properties":{"phases":{"type":"array","minItems":1,"maxItems":12,"items":{"type":"object","additionalProperties":false,"required":["title","tasks"],"properties":{"title":{"type":"string"},"tasks":{"type":"array","minItems":1,"maxItems":20,"items":{"type":"object","additionalProperties":false,"required":["title","body","acceptance"],"properties":{"title":{"type":"string"},"body":{"type":"string"},"acceptance":{"type":"string"}}}}}}}}}"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanProposal {
    pub phases: Vec<PlanPhase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanPhase {
    pub title: String,
    pub tasks: Vec<PlanTask>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanTask {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub acceptance: String,
}

impl PlanProposal {
    /// Trims the titles, then checks counts and blank titles. Used on the model's answer and
    /// again on what the user approved, because the preview is editable.
    pub fn clean(mut self) -> AppResult<Self> {
        if self.phases.is_empty() || self.phases.len() > MAX_PHASES {
            return Err(invalid(format!("the plan needs 1 to {MAX_PHASES} phases")));
        }
        for (i, phase) in self.phases.iter_mut().enumerate() {
            let n = i + 1;
            phase.title = title_of(&phase.title, &format!("phase {n}"))?;
            if phase.tasks.is_empty() || phase.tasks.len() > MAX_TASKS {
                return Err(invalid(format!("phase {n} needs 1 to {MAX_TASKS} tasks")));
            }
            for (j, task) in phase.tasks.iter_mut().enumerate() {
                task.title = title_of(&task.title, &format!("task {} of phase {n}", j + 1))?;
                task.body = task.body.trim().to_string();
                task.acceptance = task.acceptance.trim().to_string();
            }
        }
        Ok(self)
    }

    /// One task per item, tagged `phase-<n>`, with the phase named at the top of the body.
    pub fn to_task_creates(&self, project_id: &str) -> Vec<TaskCreate> {
        self.phases
            .iter()
            .enumerate()
            .flat_map(|(i, phase)| {
                let n = i + 1;
                phase.tasks.iter().map(move |task| TaskCreate {
                    project_id: project_id.to_string(),
                    parent_id: None,
                    title: task.title.clone(),
                    body_md: format!("Phase {n}: {}\n\n{}", phase.title, task.body)
                        .trim()
                        .to_string(),
                    acceptance_md: task.acceptance.clone(),
                    related_files: Vec::new(),
                    priority: Priority::Medium,
                    due_at: None,
                    tags: vec![format!("phase-{n}")],
                    recurrence: None,
                })
            })
            .collect()
    }
}

fn title_of(raw: &str, place: &str) -> AppResult<String> {
    let title = raw.trim();
    if title.is_empty() {
        return Err(invalid(format!("the title of {place} is empty")));
    }
    if title.chars().count() > MAX_TITLE {
        return Err(invalid(format!(
            "the title of {place} is longer than {MAX_TITLE} characters"
        )));
    }
    Ok(title.to_string())
}

fn invalid(message: String) -> AppError {
    AppError::Agent(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(title: &str) -> PlanTask {
        PlanTask {
            title: title.into(),
            body: " body ".into(),
            acceptance: " done when ".into(),
        }
    }

    fn proposal() -> PlanProposal {
        PlanProposal {
            phases: vec![
                PlanPhase {
                    title: "Setup".into(),
                    tasks: vec![task("Add the table"), task("Add the command")],
                },
                PlanPhase {
                    title: "Screens".into(),
                    tasks: vec![task("Build the page")],
                },
            ],
        }
    }

    #[test]
    fn the_schema_is_valid_json() {
        serde_json::from_str::<serde_json::Value>(SCHEMA).expect("schema parses");
    }

    #[test]
    fn unknown_fields_are_refused() {
        let json = r#"{"phases":[{"title":"A","tasks":[],"extra":1}]}"#;
        assert!(serde_json::from_str::<PlanProposal>(json).is_err());
    }

    #[test]
    fn clean_trims_text() {
        let cleaned = proposal().clean().expect("valid");
        assert_eq!(cleaned.phases[0].tasks[0].body, "body");
        assert_eq!(cleaned.phases[0].tasks[0].acceptance, "done when");
    }

    #[test]
    fn clean_refuses_empty_and_blank_parts() {
        assert!(PlanProposal { phases: vec![] }.clean().is_err());

        let mut blank_phase = proposal();
        blank_phase.phases[0].title = "  ".into();
        assert!(blank_phase.clean().is_err());

        let mut no_tasks = proposal();
        no_tasks.phases[1].tasks.clear();
        assert!(no_tasks.clean().is_err());

        let mut blank_task = proposal();
        blank_task.phases[0].tasks[1].title = String::new();
        let message = blank_task.clean().expect_err("blank").to_string();
        assert!(message.contains("task 2 of phase 1"));
    }

    #[test]
    fn clean_refuses_too_many_and_too_long() {
        let mut long = proposal();
        long.phases[0].tasks[0].title = "x".repeat(MAX_TITLE + 1);
        assert!(long.clean().is_err());

        let many = PlanProposal {
            phases: (0..=MAX_PHASES)
                .map(|i| PlanPhase {
                    title: format!("P{i}"),
                    tasks: vec![task("t")],
                })
                .collect(),
        };
        assert!(many.clean().is_err());
    }

    #[test]
    fn tasks_are_tagged_by_phase_in_order() {
        let creates = proposal().clean().expect("valid").to_task_creates("p1");
        let tags: Vec<_> = creates.iter().map(|c| c.tags[0].as_str()).collect();
        assert_eq!(tags, ["phase-1", "phase-1", "phase-2"]);
        assert_eq!(creates[0].project_id, "p1");
        assert_eq!(creates[2].body_md, "Phase 2: Screens\n\nbody");
        assert_eq!(creates[2].acceptance_md, "done when");
    }
}
