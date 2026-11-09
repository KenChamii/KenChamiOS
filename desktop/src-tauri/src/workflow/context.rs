use crate::db::{Db, assets, docs, projects, save_points, tasks, time_entries};
use crate::domain::portfolio::PortfolioCtx;
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

/// The project context plus finished tasks, timeline, time spent and assets.
pub async fn portfolio_context(db: &Db, project_id: &str) -> AppResult<RenderContext> {
    let mut ctx = project_context(db, project_id).await?;
    let done_tasks: Vec<_> = tasks::list(db, project_id)
        .await?
        .into_iter()
        .filter(|t| t.parent_id.is_none() && t.status == TaskStatus::Done)
        .collect();
    let points = save_points::list(db, project_id).await?;
    let entries = time_entries::list(db, project_id).await?;
    let project_assets = assets::list(db, project_id).await?;
    ctx.portfolio = Some(PortfolioCtx::build(
        &done_tasks,
        &points,
        &entries,
        &project_assets,
    ));
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
    use crate::db::templates;
    use crate::domain::ProjectInput;
    use crate::domain::SavePointInput;
    use crate::domain::TemplateKind;
    use crate::domain::asset::AssetKind;
    use crate::domain::render::render;
    use crate::domain::time_entry::{EntryKind, TimeEntryInput};

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
        move_to(&db, &finished.id, TaskStatus::Done, None, None)
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

    #[tokio::test]
    async fn portfolio_page_collects_everything_about_a_finished_project() {
        let db = Db::in_memory().await;
        let project = projects::create(
            &db,
            ProjectInput {
                name: "Demo App".to_string(),
                description: "A small demo.".to_string(),
                repo_path: None,
                color: None,
                tags: vec!["rust".to_string(), "tauri".to_string()],
                archived: false,
            },
        )
        .await
        .expect("project")
        .project;
        docs::upsert(&db, &project.id, DocKind::Idea, "Idea", "Notes get lost.")
            .await
            .expect("idea");
        docs::upsert(
            &db,
            &project.id,
            DocKind::Plan,
            "Plan",
            "Keep them in one app.",
        )
        .await
        .expect("plan");
        let shipped = create(&db, create_input(&project.id, "Ship the editor"))
            .await
            .expect("task");
        move_to(&db, &shipped.id, TaskStatus::Done, None, None)
            .await
            .expect("move");
        create(&db, create_input(&project.id, "Still open"))
            .await
            .expect("task");
        for (note, created_at) in [("Wrapped up", 1_794_182_400_000_i64), ("Started", 0)] {
            let input = SavePointInput {
                note: note.to_string(),
                done_md: String::new(),
                next_md: String::new(),
            };
            let point = save_points::create(&db, &project.id, input, None, None)
                .await
                .expect("save point");
            sqlx::query("UPDATE save_points SET created_at = ? WHERE id = ?")
                .bind(created_at)
                .bind(point.id)
                .execute(&db.pool)
                .await
                .expect("set date");
        }
        time_entries::create_manual(
            &db,
            &project.id,
            TimeEntryInput {
                task_id: None,
                started_at: 0,
                duration_s: 90 * 60,
                note: String::new(),
            },
        )
        .await
        .expect("manual entry");
        let mut conn = db.pool.acquire().await.expect("conn");
        time_entries::insert(
            &mut conn,
            &db,
            &time_entries::NewEntry {
                project_id: &project.id,
                task_id: None,
                kind: EntryKind::Focus,
                started_at: 0,
                ended_at: 25 * 60_000,
                duration_s: 25 * 60,
                note: "",
            },
        )
        .await
        .expect("focus entry");
        drop(conn);
        assets::add(&db, &project.id, AssetKind::Screenshot, "abc.png", "Editor")
            .await
            .expect("screenshot");
        assets::add(
            &db,
            &project.id,
            AssetKind::Link,
            "https://demo.test",
            "Demo",
        )
        .await
        .expect("link");
        assets::add(&db, &project.id, AssetKind::Link, "https://repo.test", "")
            .await
            .expect("link");
        templates::ensure_defaults(&db).await.expect("seed");
        let template = templates::get_default(&db, TemplateKind::Portfolio)
            .await
            .expect("template");

        let ctx = portfolio_context(&db, &project.id).await.expect("context");
        let page = render(&template.body, &ctx).expect("render");

        assert_eq!(
            page,
            "# Demo App

A small demo.

Tech stack: rust, tauri

## Problem

Notes get lost.

## Solution

Keep them in one app.

## Key features

- Ship the editor

## Timeline

- 1970-01-01 (idea): Started
- 2026-11-09 (idea): Wrapped up

## Time spent

- Total: 1h 55m
- Focus sessions: 1

## Screenshots

![Editor](images/abc.png)

## Links

- [Demo](https://demo.test)
- [https://repo.test](https://repo.test)

## What I learned

Add what you learned here."
        );
    }

    #[tokio::test]
    async fn portfolio_page_of_a_bare_project_has_only_the_basics() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Bare").await;
        templates::ensure_defaults(&db).await.expect("seed");
        let template = templates::get_default(&db, TemplateKind::Portfolio)
            .await
            .expect("template");

        let ctx = portfolio_context(&db, &project.id).await.expect("context");
        let page = render(&template.body, &ctx).expect("render");

        assert_eq!(
            page,
            "# Bare

## What I learned

Add what you learned here."
        );
    }
}
