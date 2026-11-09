use super::portfolio::PortfolioCtx;
use super::{Project, SavePoint, Task};
use crate::error::AppResult;
use crate::git::GitInfo;
use minijinja::value::Serde;
use minijinja::{Environment, UndefinedBehavior, Value};
use serde::Serialize;
use std::collections::BTreeMap;

// names the app fills itself; anything else in a prompt is asked from the user
const KNOWN_VARIABLES: [&str; 7] = [
    "project",
    "docs",
    "tasks",
    "task",
    "save_point",
    "git",
    "portfolio",
];

#[derive(Debug, Default, Serialize)]
pub struct RenderContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectCtx>,
    pub docs: DocsCtx,
    pub tasks: Vec<TaskCtx>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskCtx>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_point: Option<SavePointCtx>,
    pub git: GitCtx,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portfolio: Option<PortfolioCtx>,
    /// user supplied values for the unknown variables of a prompt
    #[serde(flatten)]
    pub variables: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectCtx {
    pub name: String,
    pub description: String,
    pub stage: &'static str,
    pub repo_path: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct DocsCtx {
    pub idea: String,
    pub plan: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskCtx {
    pub title: String,
    pub body: String,
    pub acceptance: String,
    pub related_files: Vec<String>,
    pub status: &'static str,
    pub priority: &'static str,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SavePointCtx {
    pub note: String,
    pub done: String,
    pub next: String,
}

/// Empty strings when the repo is missing, so templates can test them.
#[derive(Debug, Default, Serialize)]
pub struct GitCtx {
    pub branch: String,
    pub commit: String,
    pub dirty_files: usize,
}

impl From<&Project> for ProjectCtx {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            description: project.description.clone(),
            stage: project.stage.as_str(),
            repo_path: project.repo_path.clone().unwrap_or_default(),
            tags: project.tags.0.clone(),
        }
    }
}

impl From<&Task> for TaskCtx {
    fn from(task: &Task) -> Self {
        Self {
            title: task.title.clone(),
            body: task.body_md.clone(),
            acceptance: task.acceptance_md.clone(),
            related_files: task.related_files.0.clone(),
            status: task.status.as_str(),
            priority: task.priority.as_str(),
            tags: task.tags.0.clone(),
        }
    }
}

impl From<&SavePoint> for SavePointCtx {
    fn from(point: &SavePoint) -> Self {
        Self {
            note: point.note.clone(),
            done: point.done_md.clone(),
            next: point.next_md.clone(),
        }
    }
}

impl From<&GitInfo> for GitCtx {
    fn from(info: &GitInfo) -> Self {
        match info {
            GitInfo::Repo(repo) => Self {
                branch: repo.branch.clone().unwrap_or_default(),
                commit: repo.head.clone().unwrap_or_default(),
                dirty_files: repo.dirty_files,
            },
            _ => Self::default(),
        }
    }
}

impl RenderContext {
    /// Made-up data for the template preview in settings.
    pub fn sample() -> Self {
        let task = TaskCtx {
            title: "Add login form".into(),
            body: "Email and password fields with validation.".into(),
            acceptance: "- shows an error on a wrong password\n- keeps the session after restart"
                .into(),
            related_files: vec!["src/login.tsx".into(), "src/auth.ts".into()],
            status: "doing",
            priority: "high",
            tags: vec![],
        };
        Self {
            project: Some(ProjectCtx {
                name: "Sample Project".into(),
                description: "A small app used to preview templates.".into(),
                stage: "build",
                repo_path: "C:/code/sample".into(),
                tags: vec!["demo".into()],
            }),
            docs: DocsCtx {
                idea: "Track things in one place.".into(),
                plan: "1. Build the core\n2. Ship it".into(),
            },
            tasks: vec![task.clone()],
            task: Some(task),
            save_point: Some(SavePointCtx {
                note: "Stopped after the auth work.".into(),
                done: "- session storage".into(),
                next: "- login form".into(),
            }),
            git: GitCtx {
                branch: "main".into(),
                commit: "0123456789abcdef".into(),
                dirty_files: 2,
            },
            portfolio: Some(PortfolioCtx::sample()),
            variables: BTreeMap::new(),
        }
    }
}

fn environment() -> Environment<'static> {
    let mut env = Environment::new();
    // a missing doc or task renders as nothing instead of failing the whole template
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    env
}

pub fn render(body: &str, ctx: &RenderContext) -> AppResult<String> {
    Ok(environment().render_str(body, Value::from(Serde(ctx)))?)
}

/// Variables a template uses that the app does not provide, sorted.
pub fn unknown_variables(body: &str) -> AppResult<Vec<String>> {
    let env = environment();
    let template = env.template_from_str(body)?;
    let mut names: Vec<String> = template
        .undeclared_variables(false)
        .into_iter()
        .filter(|name| !KNOWN_VARIABLES.contains(&name.as_str()))
        .collect();
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TemplateKind;

    fn shipped(kind: TemplateKind) -> &'static str {
        kind.default_body()
    }

    #[test]
    fn brief_for_the_sample_project() {
        let out = render(shipped(TemplateKind::Brief), &RenderContext::sample()).expect("render");
        assert_eq!(
            out,
            "# Sample Project

A small app used to preview templates.

Stage: build, branch `main`

## Idea

Track things in one place.

## Plan

1. Build the core
2. Ship it

## Open tasks

- Add login form (doing, high)

  Acceptance criteria:

  - shows an error on a wrong password
  - keeps the session after restart

## Last save point

Stopped after the auth work.

Done:

- session storage

Next:

- login form
"
        );
    }

    #[test]
    fn brief_for_a_bare_project() {
        let ctx = RenderContext {
            project: RenderContext::sample().project,
            ..RenderContext::default()
        };
        let out = render(shipped(TemplateKind::Brief), &ctx).expect("render");
        assert_eq!(
            out,
            "# Sample Project

A small app used to preview templates.

Stage: build

## Open tasks

No open tasks.
"
        );
    }

    #[test]
    fn task_prompt_for_the_sample_task() {
        let out =
            render(shipped(TemplateKind::TaskPrompt), &RenderContext::sample()).expect("render");
        assert_eq!(
            out,
            "Goal: Add login form
Context: project Sample Project, stage build, branch main
Details: Email and password fields with validation.
Acceptance criteria:
- shows an error on a wrong password
- keeps the session after restart
Likely files: src/login.tsx, src/auth.ts
Constraints: follow the project brief; keep changes focused on this task.
Done when: all acceptance criteria pass and tests are green."
        );
    }

    #[test]
    fn user_variables_fill_a_prompt() {
        let ctx = RenderContext {
            variables: BTreeMap::from([("topic".to_string(), "caching".to_string())]),
            ..RenderContext::sample()
        };
        let out = render("Review {{ topic }} in {{ project.name }}", &ctx).expect("render");
        assert_eq!(out, "Review caching in Sample Project");
    }

    #[test]
    fn missing_data_renders_as_nothing() {
        let out = render(
            "[{{ project.name }}] [{{ task.title }}]",
            &RenderContext::default(),
        )
        .expect("render");
        assert_eq!(out, "[] []");
    }

    #[test]
    fn unknown_variables_skip_the_ones_the_app_provides() {
        let body = "{{ project.name }} {{ git.branch }} {{ zeta }} {{ alpha.beta }} {% for t in tasks %}{{ t.title }}{% endfor %}";
        assert_eq!(unknown_variables(body).expect("scan"), ["alpha", "zeta"]);
    }

    #[test]
    fn broken_syntax_is_a_template_error() {
        assert!(matches!(
            render("{% if %}", &RenderContext::default()),
            Err(crate::error::AppError::Template(_))
        ));
    }
}
