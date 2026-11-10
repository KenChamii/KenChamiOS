use super::handle::Db;
use crate::domain::search::{SearchHit, SearchRow, like_pattern};
use crate::error::AppResult;

const MIN_QUERY_CHARS: usize = 2;
const PER_KIND: i64 = 8;

// each query takes (pattern, limit) and leaves out deleted rows and rows of deleted projects
const QUERIES: [&str; 4] = [
    r"SELECT 'task' AS kind, t.id, t.project_id, p.name AS project_name, t.title,
             t.body_md AS body
        FROM tasks t JOIN projects p ON p.id = t.project_id
       WHERE t.deleted = 0 AND p.deleted = 0
         AND (t.title LIKE ?1 ESCAPE '\' OR t.body_md LIKE ?1 ESCAPE '\')
       ORDER BY t.status = 'done', t.updated_at DESC
       LIMIT ?2",
    r"SELECT d.kind, d.id, d.project_id, p.name AS project_name,
             CASE WHEN d.title != '' THEN d.title WHEN d.kind = 'idea' THEN 'Idea' ELSE 'Plan' END
               AS title,
             d.body_md AS body
        FROM project_docs d JOIN projects p ON p.id = d.project_id
       WHERE d.deleted = 0 AND p.deleted = 0 AND d.kind IN ('idea', 'plan')
         AND (d.title LIKE ?1 ESCAPE '\' OR d.body_md LIKE ?1 ESCAPE '\')
       ORDER BY d.updated_at DESC
       LIMIT ?2",
    r"SELECT 'prompt' AS kind, r.id, r.project_id, p.name AS project_name, r.title,
             r.body_md AS body
        FROM prompts r LEFT JOIN projects p ON p.id = r.project_id
       WHERE r.deleted = 0 AND (r.project_id IS NULL OR p.deleted = 0)
         AND (r.title LIKE ?1 ESCAPE '\' OR r.body_md LIKE ?1 ESCAPE '\')
       ORDER BY r.favorite DESC, r.updated_at DESC
       LIMIT ?2",
    r"SELECT 'save_point' AS kind, s.id, s.project_id, p.name AS project_name,
             CASE WHEN s.note != '' THEN s.note ELSE 'Save point' END AS title,
             s.next_md || char(10) || s.done_md || char(10) || s.session_summary_md AS body
        FROM save_points s JOIN projects p ON p.id = s.project_id
       WHERE s.deleted = 0 AND p.deleted = 0
         AND (s.note LIKE ?1 ESCAPE '\' OR s.done_md LIKE ?1 ESCAPE '\'
              OR s.next_md LIKE ?1 ESCAPE '\' OR s.session_summary_md LIKE ?1 ESCAPE '\')
       ORDER BY s.created_at DESC
       LIMIT ?2",
];

/// Tasks, idea and plan docs, prompts and save points that mention `query`.
pub async fn search(db: &Db, query: &str) -> AppResult<Vec<SearchHit>> {
    let query = query.trim();
    if query.chars().count() < MIN_QUERY_CHARS {
        return Ok(Vec::new());
    }
    let pattern = like_pattern(query);
    let mut hits = Vec::new();
    for sql in QUERIES {
        let rows: Vec<SearchRow> = sqlx::query_as(sql)
            .bind(&pattern)
            .bind(PER_KIND)
            .fetch_all(&db.pool)
            .await?;
        hits.extend(rows.into_iter().map(|row| SearchHit::from_row(row, query)));
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::{self, create_for_test};
    use crate::db::{docs, prompts, tasks};
    use crate::domain::search::SearchKind;
    use crate::domain::{DocKind, PromptInput};

    fn kinds(hits: &[SearchHit]) -> Vec<(SearchKind, &str)> {
        hits.iter().map(|h| (h.kind, h.title.as_str())).collect()
    }

    #[tokio::test]
    async fn finds_every_kind_by_title_or_body() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Alpha").await;
        let mut task = tasks::create_input(&project.id, "Fix login");
        task.body_md = "the oauth flow breaks".into();
        tasks::create(&db, task).await.expect("task");
        docs::upsert(&db, &project.id, DocKind::Plan, "", "Use OAuth with PKCE")
            .await
            .expect("doc");
        prompts::create(
            &db,
            PromptInput {
                project_id: None,
                title: "Review oauth code".into(),
                body_md: String::new(),
                tags: vec![],
            },
        )
        .await
        .expect("prompt");

        let hits = search(&db, "OAUTH").await.expect("search");

        assert_eq!(
            kinds(&hits),
            [
                (SearchKind::Task, "Fix login"),
                (SearchKind::Plan, "Plan"),
                (SearchKind::Prompt, "Review oauth code"),
            ]
        );
        assert_eq!(hits[0].snippet, "the oauth flow breaks");
        assert_eq!(hits[0].project_name.as_deref(), Some("Alpha"));
    }

    #[tokio::test]
    async fn skips_deleted_rows_short_queries_and_treats_wildcards_literally() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Alpha").await;
        tasks::create(&db, tasks::create_input(&project.id, "Ship 100% of it"))
            .await
            .expect("task");
        tasks::create(&db, tasks::create_input(&project.id, "Ship 1000 items"))
            .await
            .expect("task");

        assert!(search(&db, " s ").await.expect("short").is_empty());
        assert_eq!(
            kinds(&search(&db, "100%").await.expect("percent")),
            [(SearchKind::Task, "Ship 100% of it")]
        );

        projects::delete(&db, &project.id).await.expect("delete");
        assert!(search(&db, "ship").await.expect("deleted").is_empty());
    }
}
