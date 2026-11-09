use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use crate::domain::render::{RenderContext, render};
use crate::domain::{ExportTemplate, TemplateKind};
use crate::error::{AppError, AppResult};

/// Inserts the shipped template of each kind that has no active one yet.
pub async fn ensure_defaults(db: &Db) -> AppResult<()> {
    for kind in TemplateKind::ALL {
        let body = kind.default_body();
        sqlx::query(
            "INSERT INTO export_templates
                (id, created_at, updated_at, device_id, kind, name, body, is_default)
             SELECT ?, ?, ?, ?, ?, 'Default', ?, 1
              WHERE NOT EXISTS (
                SELECT 1 FROM export_templates WHERE kind = ? AND is_default = 1 AND deleted = 0)",
        )
        .bind(new_id())
        .bind(now_ms())
        .bind(db.stamp())
        .bind(&db.device_id)
        .bind(kind)
        .bind(body)
        .bind(kind)
        .execute(&db.pool)
        .await?;
    }
    Ok(())
}

pub async fn list(db: &Db) -> AppResult<Vec<ExportTemplate>> {
    let rows = sqlx::query_as(
        "SELECT * FROM export_templates WHERE deleted = 0 ORDER BY kind, created_at",
    )
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn get_default(db: &Db, kind: TemplateKind) -> AppResult<ExportTemplate> {
    sqlx::query_as(
        "SELECT * FROM export_templates WHERE kind = ? AND is_default = 1 AND deleted = 0",
    )
    .bind(kind)
    .fetch_optional(&db.pool)
    .await?
    .ok_or(AppError::NotFound("template"))
}

/// Saves a new body after checking that it renders against the sample data.
pub async fn update_body(db: &Db, id: &str, body: &str) -> AppResult<ExportTemplate> {
    render(body, &RenderContext::sample())?;
    write_body(db, id, body, "template.updated", "Edited template").await
}

/// Puts the shipped body back.
pub async fn reset(db: &Db, id: &str) -> AppResult<ExportTemplate> {
    let kind: TemplateKind =
        sqlx::query_scalar("SELECT kind FROM export_templates WHERE id = ? AND deleted = 0")
            .bind(id)
            .fetch_optional(&db.pool)
            .await?
            .ok_or(AppError::NotFound("template"))?;
    let body = kind.default_body();
    write_body(db, id, body, "template.reset", "Reset template to default").await
}

async fn write_body(
    db: &Db,
    id: &str,
    body: &str,
    kind: &str,
    summary: &str,
) -> AppResult<ExportTemplate> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE export_templates SET body = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(body)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "template")?;
    activity::record(&mut tx, db, kind, "export_template", id, summary).await?;
    tx.commit().await?;
    let template = sqlx::query_as("SELECT * FROM export_templates WHERE id = ?")
        .bind(id)
        .fetch_one(&db.pool)
        .await?;
    Ok(template)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;

    async fn seeded() -> Db {
        let db = Db::in_memory().await;
        ensure_defaults(&db).await.expect("seed");
        db
    }

    #[tokio::test]
    async fn seeds_each_default_once() {
        let db = seeded().await;
        ensure_defaults(&db).await.expect("seed again");

        let kinds: Vec<TemplateKind> = list(&db)
            .await
            .expect("list")
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                TemplateKind::Brief,
                TemplateKind::Portfolio,
                TemplateKind::TaskPrompt
            ]
        );
    }

    #[tokio::test]
    async fn edit_then_reset_restores_the_shipped_body() {
        let db = seeded().await;
        let brief = get_default(&db, TemplateKind::Brief).await.expect("brief");

        let edited = update_body(&db, &brief.id, "Hello {{ project.name }}")
            .await
            .expect("edit");
        assert_eq!(edited.body, "Hello {{ project.name }}");

        let restored = reset(&db, &brief.id).await.expect("reset");
        assert_eq!(restored.body, TemplateKind::Brief.default_body());
        assert_eq!(
            kinds_for(&db, &brief.id).await,
            ["template.updated", "template.reset"]
        );
    }

    #[tokio::test]
    async fn broken_template_is_rejected_and_not_saved() {
        let db = seeded().await;
        let brief = get_default(&db, TemplateKind::Brief).await.expect("brief");

        let result = update_body(&db, &brief.id, "{% if %}").await;

        assert!(matches!(result, Err(AppError::Template(_))));
        let unchanged = get_default(&db, TemplateKind::Brief).await.expect("brief");
        assert_eq!(unchanged.body, brief.body);
    }
}
