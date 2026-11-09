use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use crate::domain::{ProjectFilter, ProjectInput, ProjectSummary, Stage, slugify, tags};
use crate::error::{AppError, AppResult};
use sqlx::SqliteConnection;
use sqlx::types::Json;

// activity on a project's tasks and docs counts as activity on the project
macro_rules! summary_sql {
    ($tail:literal) => {
        concat!(
            "SELECT p.*,
                (SELECT COUNT(*) FROM tasks t
                  WHERE t.project_id = p.id AND t.parent_id IS NULL AND t.deleted = 0)
                    AS tasks_total,
                (SELECT COUNT(*) FROM tasks t
                  WHERE t.project_id = p.id AND t.parent_id IS NULL AND t.deleted = 0
                    AND t.status = 'done') AS tasks_done,
                COALESCE((SELECT MAX(a.at) FROM activity a
                           WHERE a.entity_id = p.id
                              OR a.entity_id IN (SELECT id FROM tasks WHERE project_id = p.id)
                              OR a.entity_id IN
                                 (SELECT id FROM project_docs WHERE project_id = p.id)),
                         p.created_at) AS last_activity_at
             FROM projects p ",
            $tail
        )
    };
}

pub async fn list(db: &Db, filter: &ProjectFilter) -> AppResult<Vec<ProjectSummary>> {
    let search = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));
    let rows = sqlx::query_as(summary_sql!(
        "WHERE p.deleted = 0
           AND (?1 OR p.archived = 0)
           AND (?2 IS NULL OR p.name LIKE ?2 OR p.description LIKE ?2)
           AND (?3 IS NULL OR EXISTS (SELECT 1 FROM json_each(p.tags) WHERE value = ?3))
         ORDER BY last_activity_at DESC"
    ))
    .bind(filter.include_archived)
    .bind(search)
    .bind(filter.tag.as_deref())
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<ProjectSummary> {
    sqlx::query_as(summary_sql!("WHERE p.id = ? AND p.deleted = 0"))
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("project"))
}

pub async fn create(db: &Db, input: ProjectInput) -> AppResult<ProjectSummary> {
    let name = clean_name(&input.name)?;
    let id = new_id();
    let mut tx = db.pool.begin().await?;
    let slug = unique_slug(&mut tx, &name).await?;
    sqlx::query(
        "INSERT INTO projects
            (id, created_at, updated_at, device_id, name, slug, description,
             repo_path, color, tags, archived)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(&name)
    .bind(slug)
    .bind(input.description.trim())
    .bind(input.repo_path)
    .bind(input.color)
    .bind(Json(tags::clean(input.tags)))
    .bind(input.archived)
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "project.created",
        "project",
        &id,
        &format!("Created project {name}"),
    )
    .await?;
    tx.commit().await?;
    get(db, &id).await
}

pub async fn update(db: &Db, id: &str, input: ProjectInput) -> AppResult<ProjectSummary> {
    let name = clean_name(&input.name)?;
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE projects
            SET name = ?, description = ?, repo_path = ?, color = ?, tags = ?, archived = ?,
                updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(&name)
    .bind(input.description.trim())
    .bind(input.repo_path)
    .bind(input.color)
    .bind(Json(tags::clean(input.tags)))
    .bind(input.archived)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "project")?;
    activity::record(
        &mut tx,
        db,
        "project.updated",
        "project",
        id,
        &format!("Updated project {name}"),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

pub async fn set_stage(db: &Db, id: &str, stage: Stage) -> AppResult<ProjectSummary> {
    let now = now_ms();
    let finished_at = (stage == Stage::Done).then_some(now);
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE projects SET stage = ?, finished_at = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(stage)
    .bind(finished_at)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "project")?;
    activity::record(
        &mut tx,
        db,
        "project.stage_changed",
        "project",
        id,
        &format!("Moved to {}", stage.as_str()),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

/// Soft deletes the project with its docs, tasks and checklist items.
pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE projects SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "project")?;
    for sql in [
        "UPDATE project_docs SET deleted = 1, updated_at = ?, device_id = ?
          WHERE project_id = ? AND deleted = 0",
        "UPDATE tasks SET deleted = 1, updated_at = ?, device_id = ?
          WHERE project_id = ? AND deleted = 0",
        "UPDATE checklist_items SET deleted = 1, updated_at = ?, device_id = ?
          WHERE deleted = 0 AND task_id IN (SELECT id FROM tasks WHERE project_id = ?)",
    ] {
        sqlx::query(sql)
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    activity::record(
        &mut tx,
        db,
        "project.deleted",
        "project",
        id,
        "Deleted project",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

fn clean_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("project name is required"));
    }
    Ok(name.to_string())
}

async fn unique_slug(conn: &mut SqliteConnection, name: &str) -> AppResult<String> {
    let base = slugify(name);
    let mut candidate = base.clone();
    let mut suffix = 2;
    while sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM projects WHERE slug = ? AND deleted = 0)",
    )
    .bind(&candidate)
    .fetch_one(&mut *conn)
    .await?
    {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    Ok(candidate)
}

#[cfg(test)]
pub async fn create_for_test(db: &Db, name: &str) -> crate::domain::project::Project {
    create(
        db,
        ProjectInput {
            name: name.to_string(),
            description: String::new(),
            repo_path: None,
            color: None,
            tags: vec![],
            archived: false,
        },
    )
    .await
    .expect("create project")
    .project
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;

    fn input(name: &str, tags: &[&str]) -> ProjectInput {
        ProjectInput {
            name: name.to_string(),
            description: "about".to_string(),
            repo_path: Some("C:/repo".to_string()),
            color: Some("#336699".to_string()),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            archived: false,
        }
    }

    #[tokio::test]
    async fn create_stores_fields_and_records_activity() {
        let db = Db::in_memory().await;
        let created = create(&db, input("  My App ", &["rust", " rust ", ""]))
            .await
            .expect("create");

        assert_eq!(created.project.name, "My App");
        assert_eq!(created.project.slug, "my-app");
        assert_eq!(created.project.stage, Stage::Idea);
        assert_eq!(created.project.tags.0, vec!["rust"]);
        assert_eq!(created.tasks_total, 0);
        assert_eq!(
            kinds_for(&db, &created.project.id).await,
            ["project.created"]
        );
    }

    #[tokio::test]
    async fn create_rejects_blank_name() {
        let db = Db::in_memory().await;
        let result = create(&db, input("   ", &[])).await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn slugs_stay_unique() {
        let db = Db::in_memory().await;
        let first = create(&db, input("Same", &[])).await.expect("first");
        let second = create(&db, input("Same", &[])).await.expect("second");
        assert_eq!(first.project.slug, "same");
        assert_eq!(second.project.slug, "same-2");
    }

    #[tokio::test]
    async fn update_changes_fields_and_stamps_row() {
        let db = Db::in_memory().await;
        let created = create(&db, input("Old", &[])).await.expect("create");
        let id = &created.project.id;
        let before: i64 = sqlx::query_scalar("SELECT updated_at FROM projects WHERE id = ?")
            .bind(id)
            .fetch_one(&db.pool)
            .await
            .expect("read stamp");

        let mut changed = input("New", &["ui"]);
        changed.archived = true;
        let updated = update(&db, id, changed).await.expect("update");

        let (after, device): (i64, String) =
            sqlx::query_as("SELECT updated_at, device_id FROM projects WHERE id = ?")
                .bind(id)
                .fetch_one(&db.pool)
                .await
                .expect("read stamp");
        assert_eq!(updated.project.name, "New");
        assert!(updated.project.archived);
        assert!(after > before);
        assert_eq!(device, "test-device");
        assert_eq!(
            kinds_for(&db, id).await,
            ["project.created", "project.updated"]
        );
    }

    #[tokio::test]
    async fn update_unknown_project_is_not_found() {
        let db = Db::in_memory().await;
        let result = update(&db, "missing", input("x", &[])).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn set_stage_tracks_finish_time() {
        let db = Db::in_memory().await;
        let id = create(&db, input("P", &[]))
            .await
            .expect("create")
            .project
            .id;

        let done = set_stage(&db, &id, Stage::Done).await.expect("done");
        assert_eq!(done.project.stage, Stage::Done);
        assert!(done.project.finished_at.is_some());

        let reopened = set_stage(&db, &id, Stage::Build).await.expect("reopen");
        assert_eq!(reopened.project.finished_at, None);
        assert_eq!(
            kinds_for(&db, &id).await,
            [
                "project.created",
                "project.stage_changed",
                "project.stage_changed"
            ]
        );
    }

    #[tokio::test]
    async fn delete_is_soft_and_hides_project() {
        let db = Db::in_memory().await;
        let id = create(&db, input("P", &[]))
            .await
            .expect("create")
            .project
            .id;

        delete(&db, &id).await.expect("delete");

        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE deleted = 1")
            .fetch_one(&db.pool)
            .await
            .expect("count");
        assert_eq!(rows, 1);
        assert!(matches!(get(&db, &id).await, Err(AppError::NotFound(_))));
        assert!(
            list(&db, &ProjectFilter::default())
                .await
                .expect("list")
                .is_empty()
        );
        assert!(matches!(delete(&db, &id).await, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn list_hides_archived_and_filters_by_search_and_tag() {
        let db = Db::in_memory().await;
        create(&db, input("Alpha", &["rust"])).await.expect("alpha");
        create(&db, input("Beta", &["web"])).await.expect("beta");
        let mut archived = input("Gamma", &["rust"]);
        archived.archived = true;
        create(&db, archived).await.expect("gamma");

        let names = |rows: Vec<ProjectSummary>| {
            let mut names: Vec<String> = rows.into_iter().map(|r| r.project.name).collect();
            names.sort();
            names
        };

        let default = list(&db, &ProjectFilter::default()).await.expect("list");
        assert_eq!(names(default), ["Alpha", "Beta"]);

        let with_archived = ProjectFilter {
            include_archived: true,
            ..ProjectFilter::default()
        };
        let all = list(&db, &with_archived).await.expect("list");
        assert_eq!(names(all), ["Alpha", "Beta", "Gamma"]);

        let by_tag = ProjectFilter {
            tag: Some("rust".to_string()),
            ..ProjectFilter::default()
        };
        assert_eq!(names(list(&db, &by_tag).await.expect("list")), ["Alpha"]);

        let by_search = ProjectFilter {
            search: Some("bet".to_string()),
            ..ProjectFilter::default()
        };
        assert_eq!(names(list(&db, &by_search).await.expect("list")), ["Beta"]);
    }
}
