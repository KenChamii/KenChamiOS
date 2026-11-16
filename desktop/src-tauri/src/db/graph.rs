use super::{Db, projects};
use crate::domain::project::{INBOX_ID, ProjectFilter};
use crate::error::AppResult;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    /// `note`, `project`, `daily` or `tag`
    pub kind: &'static str,
    pub label: String,
    /// the note's path inside the vault, for opening it in Obsidian
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, sqlx::FromRow)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

fn kind_of(id: &str) -> &'static str {
    match id.split(':').next() {
        Some("project") => "project",
        Some("daily") => "daily",
        Some("tag") => "tag",
        _ => "note",
    }
}

/// Every note, every project, and the days and tags something links to, with all the links.
pub async fn data(db: &Db) -> AppResult<GraphData> {
    let mut nodes: BTreeMap<String, GraphNode> = BTreeMap::new();
    let notes: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id, title, path FROM vault_notes")
            .fetch_all(&db.pool)
            .await?;
    for (id, title, path) in notes {
        nodes.insert(
            id.clone(),
            GraphNode {
                id,
                kind: "note",
                label: title,
                path: Some(path),
            },
        );
    }
    for summary in projects::list(db, &ProjectFilter::default()).await? {
        let project = summary.project;
        if project.id == INBOX_ID || project.archived {
            continue;
        }
        let id = format!("project:{}", project.id);
        nodes.insert(
            id.clone(),
            GraphNode {
                id,
                kind: "project",
                label: project.name,
                path: None,
            },
        );
    }
    let edges: Vec<GraphEdge> =
        sqlx::query_as("SELECT source_id AS source, target_id AS target, kind FROM links")
            .fetch_all(&db.pool)
            .await?;
    // only days and tags are made up by a link; a note or project must exist already
    for id in edges.iter().flat_map(|e| [&e.source, &e.target]) {
        if matches!(kind_of(id), "note" | "project") {
            continue;
        }
        nodes.entry(id.clone()).or_insert_with(|| GraphNode {
            id: id.clone(),
            kind: kind_of(id),
            label: id
                .split_once(':')
                .map_or(id.clone(), |(_, rest)| rest.to_string()),
            path: None,
        });
    }
    // an edge to a project that is archived or gone has nothing to draw
    let edges = edges
        .into_iter()
        .filter(|e| nodes.contains_key(&e.source) && nodes.contains_key(&e.target))
        .collect();
    Ok(GraphData {
        nodes: nodes.into_values().collect(),
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;

    #[tokio::test]
    async fn nodes_cover_notes_projects_and_link_ends() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        sqlx::query(
            "INSERT INTO vault_notes (id, path, title, mtime) VALUES ('note:a.md', 'a.md', 'a', 1)",
        )
        .execute(&db.pool)
        .await
        .expect("note");
        for (source, target, kind) in [
            ("note:a.md".to_string(), "tag:idea".to_string(), "tag"),
            (
                "note:a.md".to_string(),
                format!("project:{}", project.id),
                "wikilink",
            ),
            (
                "daily:2026-11-16".to_string(),
                "project:gone".to_string(),
                "planned",
            ),
        ] {
            sqlx::query("INSERT INTO links VALUES (?, 'note', ?, 'tag', ?)")
                .bind(source)
                .bind(target)
                .bind(kind)
                .execute(&db.pool)
                .await
                .expect("link");
        }

        let graph = data(&db).await.expect("graph");

        let kinds: Vec<_> = graph
            .nodes
            .iter()
            .map(|n| (n.kind, n.label.as_str()))
            .collect();
        assert!(kinds.contains(&("note", "a")));
        assert!(kinds.contains(&("project", "Demo")));
        assert!(kinds.contains(&("tag", "idea")));
        assert_eq!(
            graph.edges.len(),
            2,
            "the edge to a missing project is dropped"
        );
    }
}
