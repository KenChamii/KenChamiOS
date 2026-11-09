use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use crate::domain::{Prompt, PromptInput, tags};
use crate::error::{AppError, AppResult};
use sqlx::types::Json;

/// Favorites first, then the most used.
pub async fn list(db: &Db) -> AppResult<Vec<Prompt>> {
    let rows = sqlx::query_as(
        "SELECT * FROM prompts WHERE deleted = 0
          ORDER BY favorite DESC, use_count DESC, COALESCE(last_used_at, 0) DESC, title",
    )
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<Prompt> {
    sqlx::query_as("SELECT * FROM prompts WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("prompt"))
}

pub async fn create(db: &Db, input: PromptInput) -> AppResult<Prompt> {
    let title = clean_title(&input.title)?;
    let id = new_id();
    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO prompts
            (id, created_at, updated_at, device_id, project_id, title, body_md, tags)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(&input.project_id)
    .bind(&title)
    .bind(&input.body_md)
    .bind(Json(tags::clean(input.tags)))
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "prompt.created",
        "prompt",
        &id,
        &format!("Added prompt {title}"),
    )
    .await?;
    tx.commit().await?;
    get(db, &id).await
}

pub async fn update(db: &Db, id: &str, input: PromptInput) -> AppResult<Prompt> {
    let title = clean_title(&input.title)?;
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE prompts
            SET project_id = ?, title = ?, body_md = ?, tags = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(&input.project_id)
    .bind(&title)
    .bind(&input.body_md)
    .bind(Json(tags::clean(input.tags)))
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "prompt")?;
    activity::record(
        &mut tx,
        db,
        "prompt.updated",
        "prompt",
        id,
        &format!("Updated prompt {title}"),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

pub async fn set_favorite(db: &Db, id: &str, favorite: bool) -> AppResult<Prompt> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE prompts SET favorite = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(favorite)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "prompt")?;
    let summary = if favorite {
        "Marked prompt as favorite"
    } else {
        "Removed prompt from favorites"
    };
    activity::record(&mut tx, db, "prompt.favorite", "prompt", id, summary).await?;
    tx.commit().await?;
    get(db, id).await
}

/// Counts one copy of the prompt.
pub async fn mark_used(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE prompts
            SET use_count = use_count + 1, last_used_at = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(now_ms())
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "prompt")?;
    activity::record(&mut tx, db, "prompt.used", "prompt", id, "Copied prompt").await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE prompts SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "prompt")?;
    activity::record(
        &mut tx,
        db,
        "prompt.deleted",
        "prompt",
        id,
        "Deleted prompt",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

fn clean_title(title: &str) -> AppResult<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("prompt title is required"));
    }
    Ok(title.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;
    use crate::db::projects::create_for_test;

    fn input(title: &str, project_id: Option<&str>) -> PromptInput {
        PromptInput {
            project_id: project_id.map(str::to_string),
            title: title.to_string(),
            body_md: "Review {{ project.name }}".to_string(),
            tags: vec![" review ".to_string(), "review".to_string()],
        }
    }

    #[tokio::test]
    async fn create_trims_and_logs() {
        let db = Db::in_memory().await;
        let prompt = create(&db, input(" Code review ", None))
            .await
            .expect("create");

        assert_eq!(prompt.title, "Code review");
        assert_eq!(prompt.tags.0, ["review"]);
        assert_eq!(prompt.project_id, None);
        assert_eq!(prompt.use_count, 0);
        assert_eq!(kinds_for(&db, &prompt.id).await, ["prompt.created"]);
    }

    #[tokio::test]
    async fn create_rejects_blank_title() {
        let db = Db::in_memory().await;
        let result = create(&db, input("  ", None)).await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn project_scope_is_stored_and_editable() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let prompt = create(&db, input("Scoped", Some(&project.id)))
            .await
            .expect("create");
        assert_eq!(prompt.project_id.as_deref(), Some(project.id.as_str()));

        let global = update(&db, &prompt.id, input("Scoped", None))
            .await
            .expect("update");
        assert_eq!(global.project_id, None);
    }

    #[tokio::test]
    async fn list_puts_favorites_then_most_used_first() {
        let db = Db::in_memory().await;
        let plain = create(&db, input("plain", None)).await.expect("create");
        let busy = create(&db, input("busy", None)).await.expect("create");
        let fav = create(&db, input("fav", None)).await.expect("create");
        mark_used(&db, &busy.id).await.expect("use");
        mark_used(&db, &busy.id).await.expect("use");
        mark_used(&db, &plain.id).await.expect("use");
        set_favorite(&db, &fav.id, true).await.expect("favorite");

        let titles: Vec<String> = list(&db)
            .await
            .expect("list")
            .into_iter()
            .map(|p| p.title)
            .collect();
        assert_eq!(titles, ["fav", "busy", "plain"]);
    }

    #[tokio::test]
    async fn mark_used_counts_and_stamps() {
        let db = Db::in_memory().await;
        let prompt = create(&db, input("p", None)).await.expect("create");

        mark_used(&db, &prompt.id).await.expect("use");
        mark_used(&db, &prompt.id).await.expect("use again");

        let used = get(&db, &prompt.id).await.expect("get");
        assert_eq!(used.use_count, 2);
        assert!(used.last_used_at.is_some());
    }

    #[tokio::test]
    async fn delete_hides_the_prompt() {
        let db = Db::in_memory().await;
        let prompt = create(&db, input("p", None)).await.expect("create");

        delete(&db, &prompt.id).await.expect("delete");

        assert!(list(&db).await.expect("list").is_empty());
        assert!(matches!(
            delete(&db, &prompt.id).await,
            Err(AppError::NotFound(_))
        ));
    }
}
