use crate::db::{Db, docs, projects, save_points, tasks};
use crate::domain::render::{GitCtx, ProjectCtx, RenderContext, SavePointCtx, TaskCtx};
use crate::domain::{DocKind, TaskStatus};
use crate::error::AppResult;
use crate::git;
use std::collections::BTreeMap;

/// Project, docs, open tasks, last save point and git state.
pub async fn project_context(db: &Db, project_id: &str) -> AppResult<RenderContext> {
    let project = projects::get(db, project_id).await?.project;
    let idea = docs::get(db, project_id, DocKind::Idea).await?;
    let plan = docs::get(db, project_id, DocKind::Plan).await?;
    let open_tasks = tasks::list(db, project_id)
        .await?
        .iter()
        .filter(|t| t.parent_id.is_none() && t.status != TaskStatus::Done)
        .map(TaskCtx::from)
        .collect();
    let points = save_points::list(db, project_id).await?;
    let git = git::read(project.repo_path.as_deref())?;

    let mut ctx = RenderContext {
        project: Some(ProjectCtx::from(&project)),
        tasks: open_tasks,
        save_point: points.first().map(SavePointCtx::from),
        git: GitCtx::from(&git),
        ..RenderContext::default()
    };
    ctx.docs.idea = idea.map(|d| d.body_md).unwrap_or_default();
    ctx.docs.plan = plan.map(|d| d.body_md).unwrap_or_default();
    Ok(ctx)
}

/// The project context plus one task.
pub async fn task_context(db: &Db, task_id: &str) -> AppResult<RenderContext> {
    let task = tasks::get(db, task_id).await?;
    let mut ctx = project_context(db, &task.project_id).await?;
    ctx.task = Some(TaskCtx::from(&task));
    Ok(ctx)
}

/// A prompt has project data only when a project is given.
pub async fn prompt_context(
    db: &Db,
    project_id: Option<&str>,
    variables: BTreeMap<String, String>,
) -> AppResult<RenderContext> {
    let mut ctx = match project_id {
        Some(id) => project_context(db, id).await?,
        None => RenderContext::default(),
    };
    ctx.variables = variables;
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{create, create_input, move_to};
    use crate::domain::SavePointInput;

    #[tokio::test]
    async fn collects_open_tasks_docs_and_last_save_point() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        docs::upsert(&db, &project.id, DocKind::Plan, "Plan", "Ship it")
            .await
            .expect("plan");
        create(&db, create_input(&project.id, "Open one"))
            .await
            .expect("task");
        let finished = create(&db, create_input(&project.id, "Finished"))
            .await
            .expect("task");
        move_to(&db, &finished.id, TaskStatus::Done, None)
            .await
            .expect("move");
        let note = |text: &str| SavePointInput {
            note: text.to_string(),
            done_md: String::new(),
            next_md: String::new(),
        };
        save_points::create(&db, &project.id, note("older"), None, None)
            .await
            .expect("save point");
        save_points::create(&db, &project.id, note("latest"), None, None)
            .await
            .expect("save point");

        let ctx = project_context(&db, &project.id).await.expect("context");

        let titles: Vec<&str> = ctx.tasks.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["Open one"]);
        assert_eq!(ctx.docs.plan, "Ship it");
        assert_eq!(ctx.docs.idea, "");
        assert_eq!(ctx.save_point.map(|p| p.note).as_deref(), Some("latest"));
        assert_eq!(ctx.git.branch, "");
    }

    #[tokio::test]
    async fn task_context_adds_the_task() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        let task = create(&db, create_input(&project.id, "Do it"))
            .await
            .expect("task");

        let ctx = task_context(&db, &task.id).await.expect("context");

        assert_eq!(ctx.task.map(|t| t.title).as_deref(), Some("Do it"));
        assert_eq!(ctx.project.map(|p| p.name).as_deref(), Some("Demo"));
    }
}
