use super::handle::Db;
use super::save_points::{self, NewSavePoint};
use super::{projects, tasks};
use crate::domain::session_note::SessionConfirm;
use crate::domain::task::Priority;
use crate::domain::{SavePoint, TaskCreate, TaskStatus};
use crate::error::{AppError, AppResult};

/// Saves the confirmed session note as one save point, moves the matched tasks to
/// done and adds the new ones, all in one transaction.
pub async fn confirm(
    db: &Db,
    input: &SessionConfirm,
    git_branch: Option<&str>,
    git_commit: Option<&str>,
) -> AppResult<SavePoint> {
    let stage = projects::get(db, &input.project_id).await?.project.stage;
    let done_ids = unique(&input.done_task_ids);
    for id in &done_ids {
        if tasks::get(db, id).await?.project_id != input.project_id {
            return Err(AppError::Invalid("task belongs to another project"));
        }
    }

    let done_md = done_markdown(&input.done, &input.changed_files);
    let next_md = bullets(&input.next);
    let mut tx = db.pool.begin().await?;
    let id = save_points::insert(
        &mut tx,
        db,
        &NewSavePoint {
            project_id: &input.project_id,
            stage,
            source: "session_note",
            note: &input.note,
            done_md: &done_md,
            next_md: &next_md,
            session_summary_md: &input.summary_md,
            git_branch,
            git_commit,
        },
    )
    .await?;
    for task_id in &done_ids {
        tasks::move_in(&mut tx, db, task_id, TaskStatus::Done, None).await?;
    }
    for title in &input.new_task_titles {
        let task = TaskCreate {
            project_id: input.project_id.clone(),
            parent_id: None,
            title: title.clone(),
            body_md: String::new(),
            acceptance_md: String::new(),
            related_files: vec![],
            priority: Priority::Medium,
            due_at: None,
            tags: vec![],
        };
        tasks::insert(&mut tx, db, &task).await?;
    }
    let saved = sqlx::query_as("SELECT * FROM save_points WHERE id = ?")
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(saved)
}

fn unique(ids: &[String]) -> Vec<&String> {
    let mut seen: Vec<&String> = Vec::new();
    for id in ids {
        if !seen.contains(&id) {
            seen.push(id);
        }
    }
    seen
}

fn bullets(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// the save point has no column for changed files, so they close the done list
fn done_markdown(done: &[String], changed_files: &[String]) -> String {
    let mut out = bullets(done);
    if !changed_files.is_empty() {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        let files: Vec<String> = changed_files.iter().map(|f| format!("`{f}`")).collect();
        out.push_str(&format!("Changed files:\n{}", bullets(&files)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{create_input, list};

    fn confirm_input(project_id: &str) -> SessionConfirm {
        SessionConfirm {
            project_id: project_id.to_string(),
            note: "Good session".to_string(),
            summary_md: "raw text".to_string(),
            done: vec!["login form".to_string()],
            next: vec!["deploy".to_string()],
            changed_files: vec!["src/a.ts".to_string()],
            done_task_ids: vec![],
            new_task_titles: vec![],
        }
    }

    #[tokio::test]
    async fn creates_save_point_with_task_updates() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let open = tasks::create(&db, create_input(&project.id, "Login form"))
            .await
            .expect("task");
        let mut input = confirm_input(&project.id);
        input.done_task_ids = vec![open.id.clone(), open.id.clone()];
        input.new_task_titles = vec!["Deploy".to_string()];

        let point = confirm(&db, &input, Some("main"), Some("abc"))
            .await
            .expect("confirm");

        assert_eq!(point.source, "session_note");
        assert_eq!(point.note, "Good session");
        assert_eq!(
            point.done_md,
            "- login form\n\nChanged files:\n- `src/a.ts`"
        );
        assert_eq!(point.next_md, "- deploy");
        assert_eq!(point.session_summary_md, "raw text");
        assert_eq!(point.git_commit.as_deref(), Some("abc"));
        let all = list(&db, &project.id).await.expect("tasks");
        assert_eq!(all.len(), 2);
        let status_of = |title: &str| all.iter().find(|t| t.title == title).map(|t| t.status);
        assert_eq!(status_of("Login form"), Some(TaskStatus::Done));
        assert_eq!(status_of("Deploy"), Some(TaskStatus::Todo));
        assert_eq!(kinds_for(&db, &point.id).await, ["save_point.created"]);
        assert_eq!(
            kinds_for(&db, &open.id).await,
            ["task.created", "task.moved"]
        );
    }

    #[tokio::test]
    async fn failure_rolls_everything_back() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let mut input = confirm_input(&project.id);
        input.new_task_titles = vec!["Fine".to_string(), "  ".to_string()];

        let result = confirm(&db, &input, None, None).await;

        assert!(matches!(result, Err(AppError::Invalid(_))));
        assert!(
            save_points::list(&db, &project.id)
                .await
                .expect("list")
                .is_empty()
        );
        assert!(list(&db, &project.id).await.expect("tasks").is_empty());
    }

    #[tokio::test]
    async fn rejects_tasks_of_another_project() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let other = create_for_test(&db, "Q").await;
        let foreign = tasks::create(&db, create_input(&other.id, "Elsewhere"))
            .await
            .expect("task");
        let mut input = confirm_input(&project.id);
        input.done_task_ids = vec![foreign.id];

        let result = confirm(&db, &input, None, None).await;

        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn blank_note_is_rejected() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let mut input = confirm_input(&project.id);
        input.note = " ".to_string();

        assert!(matches!(
            confirm(&db, &input, None, None).await,
            Err(AppError::Invalid(_))
        ));
    }
}
