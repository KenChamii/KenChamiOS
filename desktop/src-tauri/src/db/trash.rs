use super::activity;
use super::handle::Db;
use crate::domain::project::INBOX_ID;
use crate::domain::trash::{TrashItem, TrashKind};
use crate::error::{AppError, AppResult};

const MAX_ITEMS: i64 = 200;

/// Deleted projects, tasks, prompts, save points, media and habits, newest first.
pub async fn list(db: &Db) -> AppResult<Vec<TrashItem>> {
    // updated_at is an hlc, its upper bits are the wall clock in ms
    let rows = sqlx::query_as(
        "SELECT 'project' AS kind, p.id, p.name AS title, NULL AS detail,
                p.updated_at >> 16 AS deleted_at
           FROM projects p
          WHERE p.deleted = 1 AND p.id != ?1
         UNION ALL
         SELECT 'task', t.id, t.title, p.name, t.updated_at >> 16
           FROM tasks t JOIN projects p ON p.id = t.project_id
          WHERE t.deleted = 1 AND p.deleted = 0
            AND (t.parent_id IS NULL
                 OR EXISTS (SELECT 1 FROM tasks pt WHERE pt.id = t.parent_id AND pt.deleted = 0))
         UNION ALL
         SELECT 'prompt', r.id, r.title, p.name, r.updated_at >> 16
           FROM prompts r LEFT JOIN projects p ON p.id = r.project_id
          WHERE r.deleted = 1 AND (r.project_id IS NULL OR p.deleted = 0)
         UNION ALL
         SELECT 'save_point', s.id, CASE WHEN s.note != '' THEN s.note ELSE 'Save point' END,
                p.name, s.updated_at >> 16
           FROM save_points s JOIN projects p ON p.id = s.project_id
          WHERE s.deleted = 1 AND p.deleted = 0
         UNION ALL
         SELECT 'media', e.id, i.title, i.type, e.updated_at >> 16
           FROM media_entries e JOIN media_items i ON i.id = e.media_item_id
          WHERE e.deleted = 1
         UNION ALL
         SELECT 'habit', h.id, h.name, NULL, h.updated_at >> 16
           FROM habits h
          WHERE h.deleted = 1
         ORDER BY deleted_at DESC, id
         LIMIT ?2",
    )
    .bind(INBOX_ID)
    .bind(MAX_ITEMS)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// What one restore reads and writes; every update is bound as
/// (updated_at, device_id, id, hlc of the delete).
struct Plan {
    what: &'static str,
    entity_type: &'static str,
    activity_kind: &'static str,
    stamp_sql: &'static str,
    updates: &'static [&'static str],
}

// rows deleted at or after the item's own delete went with the same action
fn plan(kind: TrashKind) -> Plan {
    match kind {
        TrashKind::Project => Plan {
            what: "deleted project",
            entity_type: "project",
            activity_kind: "project.restored",
            stamp_sql: "SELECT updated_at FROM projects WHERE id = ? AND deleted = 1",
            updates: &[
                "UPDATE projects SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE project_docs SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE project_id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE prompts SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE project_id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE save_points SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE project_id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE project_assets SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE project_id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE time_entries SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE project_id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE checklist_items SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE task_id IN (SELECT id FROM tasks WHERE project_id = ?3)
                    AND deleted = 1 AND updated_at >= ?4",
                "UPDATE tasks SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE project_id = ?3 AND deleted = 1 AND updated_at >= ?4",
            ],
        },
        TrashKind::Task => Plan {
            what: "deleted task",
            entity_type: "task",
            activity_kind: "task.restored",
            stamp_sql: "SELECT t.updated_at FROM tasks t JOIN projects p ON p.id = t.project_id
                          WHERE t.id = ? AND t.deleted = 1 AND p.deleted = 0",
            updates: &[
                "UPDATE checklist_items SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE task_id IN (SELECT id FROM tasks WHERE id = ?3 OR parent_id = ?3)
                    AND deleted = 1 AND updated_at >= ?4",
                "UPDATE tasks SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE (id = ?3 OR parent_id = ?3) AND deleted = 1 AND updated_at >= ?4",
            ],
        },
        TrashKind::Prompt => Plan {
            what: "deleted prompt",
            entity_type: "prompt",
            activity_kind: "prompt.restored",
            stamp_sql: "SELECT updated_at FROM prompts WHERE id = ? AND deleted = 1",
            updates: &[
                "UPDATE prompts SET deleted = 0, updated_at = ?1, device_id = ?2
                         WHERE id = ?3 AND deleted = 1 AND updated_at >= ?4",
            ],
        },
        TrashKind::SavePoint => Plan {
            what: "deleted save point",
            entity_type: "save_point",
            activity_kind: "save_point.restored",
            stamp_sql: "SELECT updated_at FROM save_points WHERE id = ? AND deleted = 1",
            updates: &[
                "UPDATE save_points SET deleted = 0, updated_at = ?1, device_id = ?2
                         WHERE id = ?3 AND deleted = 1 AND updated_at >= ?4",
            ],
        },
        TrashKind::Media => Plan {
            what: "deleted library entry",
            entity_type: "media_entry",
            activity_kind: "media.restored",
            stamp_sql: "SELECT updated_at FROM media_entries WHERE id = ? AND deleted = 1",
            updates: &[
                "UPDATE media_items SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE id = (SELECT media_item_id FROM media_entries WHERE id = ?3)
                    AND deleted = 1 AND updated_at >= ?4",
                "UPDATE media_entries SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE id = ?3 AND deleted = 1 AND updated_at >= ?4",
            ],
        },
        TrashKind::Habit => Plan {
            what: "deleted habit",
            entity_type: "habit",
            activity_kind: "habit.restored",
            stamp_sql: "SELECT updated_at FROM habits WHERE id = ? AND deleted = 1",
            updates: &[
                "UPDATE habits SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE id = ?3 AND deleted = 1 AND updated_at >= ?4",
                "UPDATE habit_checks SET deleted = 0, updated_at = ?1, device_id = ?2
                  WHERE habit_id = ?3 AND deleted = 1 AND updated_at >= ?4",
            ],
        },
    }
}

/// Brings a deleted row back with whatever was deleted along with it.
/// A task comes back only while its project is live.
pub async fn restore(db: &Db, kind: TrashKind, id: &str) -> AppResult<()> {
    let plan = plan(kind);
    let mut tx = db.pool.begin().await?;
    let since: i64 = sqlx::query_scalar(plan.stamp_sql)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound(plan.what))?;
    for &sql in plan.updates {
        let result = sqlx::query(sql)
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(id)
            .bind(since)
            .execute(&mut *tx)
            .await;
        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                return Err(AppError::Invalid("a live copy of this item already exists"));
            }
            Err(error) => return Err(error.into()),
        }
    }
    activity::record(
        &mut tx,
        db,
        plan.activity_kind,
        plan.entity_type,
        id,
        "Restored from trash",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::{self, create_for_test};
    use crate::db::{checklist, prompts, tasks};
    use crate::domain::PromptInput;

    async fn trash_ids(db: &Db) -> Vec<(TrashKind, String)> {
        list(db)
            .await
            .expect("list")
            .into_iter()
            .map(|item| (item.kind, item.id))
            .collect()
    }

    #[tokio::test]
    async fn a_task_comes_back_with_its_subtasks_but_not_with_older_deletes() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let parent = tasks::create(&db, tasks::create_input(&project.id, "Parent"))
            .await
            .expect("parent");
        let mut early = tasks::create_input(&project.id, "Deleted earlier");
        early.parent_id = Some(parent.id.clone());
        let early = tasks::create(&db, early).await.expect("early");
        let mut sub = tasks::create_input(&project.id, "Sub");
        sub.parent_id = Some(parent.id.clone());
        let sub = tasks::create(&db, sub).await.expect("sub");
        let item = checklist::create(&db, &parent.id, "step")
            .await
            .expect("item");
        tasks::delete(&db, &early.id).await.expect("delete early");
        tasks::delete(&db, &parent.id).await.expect("delete parent");

        assert_eq!(
            trash_ids(&db).await,
            [(TrashKind::Task, parent.id.clone())],
            "subtasks of a deleted parent are not listed"
        );
        restore(&db, TrashKind::Task, &parent.id)
            .await
            .expect("restore");

        let live: Vec<String> = tasks::list(&db, &project.id)
            .await
            .expect("list")
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert!(live.contains(&parent.id) && live.contains(&sub.id));
        assert!(!live.contains(&early.id));
        assert_eq!(
            checklist::list(&db, &parent.id).await.expect("items")[0].id,
            item.id
        );
        assert_eq!(trash_ids(&db).await, [(TrashKind::Task, early.id)]);
        assert_eq!(
            kinds_for(&db, &parent.id).await,
            [
                "task.created",
                "checklist.created",
                "task.deleted",
                "task.restored"
            ]
        );
    }

    #[tokio::test]
    async fn a_project_comes_back_with_everything_its_delete_took() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let task = tasks::create(&db, tasks::create_input(&project.id, "Task"))
            .await
            .expect("task");
        let prompt = prompts::create(
            &db,
            PromptInput {
                project_id: Some(project.id.clone()),
                title: "Prompt".into(),
                body_md: String::new(),
                tags: vec![],
            },
        )
        .await
        .expect("prompt");
        projects::delete(&db, &project.id).await.expect("delete");
        assert_eq!(
            trash_ids(&db).await,
            [(TrashKind::Project, project.id.clone())]
        );
        assert!(matches!(
            restore(&db, TrashKind::Task, &task.id).await,
            Err(AppError::NotFound(_))
        ));

        restore(&db, TrashKind::Project, &project.id)
            .await
            .expect("restore");

        assert!(projects::get(&db, &project.id).await.is_ok());
        assert!(tasks::get(&db, &task.id).await.is_ok());
        assert_eq!(
            prompts::get(&db, &prompt.id).await.expect("prompt").id,
            prompt.id
        );
        assert!(trash_ids(&db).await.is_empty());
        assert!(matches!(
            restore(&db, TrashKind::Project, &project.id).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn restoring_bumps_the_hlc_so_sync_sends_it() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let task = tasks::create(&db, tasks::create_input(&project.id, "Task"))
            .await
            .expect("task");
        tasks::delete(&db, &task.id).await.expect("delete");
        let deleted_at: i64 = sqlx::query_scalar("SELECT updated_at FROM tasks WHERE id = ?")
            .bind(&task.id)
            .fetch_one(&db.pool)
            .await
            .expect("stamp");

        restore(&db, TrashKind::Task, &task.id)
            .await
            .expect("restore");

        let (restored_at, logged): (i64, i64) = sqlx::query_as(
            "SELECT t.updated_at, l.hlc FROM tasks t
               JOIN change_log l ON l.table_name = 'tasks' AND l.row_id = t.id
              WHERE t.id = ?",
        )
        .bind(&task.id)
        .fetch_one(&db.pool)
        .await
        .expect("row");
        assert!(restored_at > deleted_at);
        assert_eq!(restored_at, logged);
    }
}
