use super::dates::{day_string, days_of, parse_day};
use super::notes::{self, ParsedNote};
use super::specs::APP_DIR;
use crate::db::{Db, planner, projects};
use crate::domain::project::{INBOX_ID, ProjectFilter};
use crate::error::AppResult;
use serde::Serialize;
use sqlx::types::Json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 20_000;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const PLAN_LOOKBACK_DAYS: i64 = 60;
const PLAN_LOOKAHEAD_DAYS: i64 = 14;

/// A note of the vault that the app does not own.
struct Note {
    id: String,
    path: String,
    title: String,
    mtime: i64,
    parsed: ParsedNote,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IndexStats {
    pub notes: usize,
    pub links: usize,
}

fn read_notes(root: &Path) -> Vec<Note> {
    let mut found = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                // .obsidian, .trash and friends, and the folder the app writes itself
                if !name.starts_with('.') && name != APP_DIR {
                    folders.push(path);
                }
            } else if path.extension().is_some_and(|ext| ext == "md")
                && found.len() < MAX_FILES
                && let Some(note) = read_note(root, &path)
            {
                found.push(note);
            }
        }
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found
}

fn read_note(root: &Path, path: &PathBuf) -> Option<Note> {
    let meta = fs::metadata(path).ok()?;
    if meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let text = fs::read_to_string(path).ok()?;
    let relative = path
        .strip_prefix(root)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    Some(Note {
        id: format!("note:{relative}"),
        title: path.file_stem()?.to_string_lossy().into_owned(),
        path: relative,
        mtime: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0)),
        parsed: notes::parse(&text),
    })
}

type Link = (String, &'static str, String, &'static str, &'static str);

fn tag_id(tag: &str) -> String {
    format!("tag:{}", tag.to_lowercase())
}

/// Rebuilds the note index and every link from the vault and the app's own records.
pub async fn refresh(db: &Db, root: &str, now: i64) -> AppResult<IndexStats> {
    let found = read_notes(Path::new(root.trim()));
    let project_list = projects::list(db, &ProjectFilter::default()).await?;
    let projects: Vec<_> = project_list
        .into_iter()
        .map(|s| s.project)
        .filter(|p| p.id != INBOX_ID && !p.archived)
        .collect();

    let by_title: HashMap<String, &Note> = found
        .iter()
        .rev()
        .map(|note| (note.title.to_lowercase(), note))
        .collect();
    let by_project: HashMap<String, &str> = projects
        .iter()
        .map(|p| (p.name.to_lowercase(), p.id.as_str()))
        .collect();

    let mut links: Vec<Link> = Vec::new();
    for note in &found {
        for name in &note.parsed.links {
            let key = name.to_lowercase();
            if let Some(target) = by_title.get(&key)
                && target.id != note.id
            {
                links.push((
                    note.id.clone(),
                    "note",
                    target.id.clone(),
                    "note",
                    "wikilink",
                ));
            } else if let Some(id) = by_project.get(&key) {
                links.push((
                    note.id.clone(),
                    "note",
                    format!("project:{id}"),
                    "project",
                    "wikilink",
                ));
            } else if parse_day(name).is_some() {
                links.push((
                    note.id.clone(),
                    "note",
                    format!("daily:{name}"),
                    "daily",
                    "wikilink",
                ));
            }
        }
        for tag in &note.parsed.tags {
            links.push((note.id.clone(), "note", tag_id(tag), "tag", "tag"));
        }
    }
    for project in &projects {
        for tag in project.tags.iter() {
            links.push((
                format!("project:{}", project.id),
                "project",
                tag_id(tag),
                "tag",
                "tag",
            ));
        }
    }
    let today = days_of(now);
    let planned = planner::planned(
        db,
        &day_string(today - PLAN_LOOKBACK_DAYS),
        &day_string(today + PLAN_LOOKAHEAD_DAYS),
    )
    .await?;
    let project_ids: HashSet<&str> = projects.iter().map(|p| p.id.as_str()).collect();
    for task in planned {
        let Some(day) = task.planned_on else { continue };
        if project_ids.contains(task.project_id.as_str()) {
            links.push((
                format!("daily:{day}"),
                "daily",
                format!("project:{}", task.project_id),
                "project",
                "planned",
            ));
        }
    }

    let mut tx = db.pool.begin().await?;
    sqlx::query("DELETE FROM vault_notes")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM links").execute(&mut *tx).await?;
    for note in &found {
        sqlx::query(
            "INSERT INTO vault_notes (id, path, title, tags, headings, mtime) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&note.id)
        .bind(&note.path)
        .bind(&note.title)
        .bind(Json(&note.parsed.tags))
        .bind(Json(&note.parsed.headings))
        .bind(note.mtime)
        .execute(&mut *tx)
        .await?;
    }
    for (source, source_kind, target, target_kind, kind) in &links {
        sqlx::query(
            "INSERT OR IGNORE INTO links (source_id, source_kind, target_id, target_kind, kind)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(source)
        .bind(source_kind)
        .bind(target)
        .bind(target_kind)
        .bind(kind)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(IndexStats {
        notes: found.len(),
        links: links.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;
    use crate::db::tasks::{self, create_input};
    use crate::domain::planner::TaskPlan;
    use crate::vault::dates::day_start_ms;

    struct Vault(PathBuf);

    impl Vault {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("kenchamios-index-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("dir");
            Self(dir)
        }

        fn write(&self, relative: &str, text: &str) {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
            fs::write(path, text).expect("write");
        }

        fn root(&self) -> &str {
            self.0.to_str().expect("utf-8")
        }
    }

    impl Drop for Vault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    async fn all_links(db: &Db) -> Vec<(String, String, String)> {
        sqlx::query_as(
            "SELECT source_id, target_id, kind FROM links ORDER BY source_id, target_id, kind",
        )
        .fetch_all(&db.pool)
        .await
        .expect("links")
    }

    const NOW: i64 = 20_777 * 86_400_000 + 12 * 3_600_000;

    #[tokio::test]
    async fn notes_link_to_notes_projects_days_and_tags() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo App").await;
        let vault = Vault::new("links");
        vault.write(
            "Alpha.md",
            "see [[beta|B]], [[Demo App]], [[2026-11-20]] #idea [[missing]]",
        );
        vault.write("sub/Beta.md", "back to [[Alpha#top]]");

        let stats = refresh(&db, vault.root(), NOW).await.expect("refresh");

        assert_eq!(stats.notes, 2);
        let links = all_links(&db).await;
        let has = |s: &str, t: &str, k: &str| links.contains(&(s.into(), t.into(), k.into()));
        assert!(has("note:Alpha.md", "note:sub/Beta.md", "wikilink"));
        assert!(has("note:sub/Beta.md", "note:Alpha.md", "wikilink"));
        assert!(has(
            "note:Alpha.md",
            &format!("project:{}", project.id),
            "wikilink"
        ));
        assert!(has("note:Alpha.md", "daily:2026-11-20", "wikilink"));
        assert!(has("note:Alpha.md", "tag:idea", "tag"));
        assert_eq!(links.len(), 5, "a missing target makes no edge: {links:?}");
    }

    #[tokio::test]
    async fn hidden_folders_and_the_app_folder_are_not_indexed() {
        let db = Db::in_memory().await;
        let vault = Vault::new("skip");
        vault.write(".obsidian/x.md", "[[a]]");
        vault.write(".trash/old.md", "x");
        vault.write("KenChamiOS/Daily/2026-11-16.md", "x");
        vault.write("Real.md", "x");

        let stats = refresh(&db, vault.root(), NOW).await.expect("refresh");

        assert_eq!(stats.notes, 1);
    }

    #[tokio::test]
    async fn projects_link_to_their_tags_and_planned_days() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "P").await;
        let task = tasks::create(&db, create_input(&project.id, "t"))
            .await
            .expect("task");
        let day = day_string(days_of(NOW));
        let plan = TaskPlan {
            planned_on: Some(day.clone()),
            scheduled_at: None,
            duration_min: None,
        };
        planner::set_plan(&db, &task.id, plan).await.expect("plan");
        let vault = Vault::new("planned");

        refresh(&db, vault.root(), NOW).await.expect("refresh");

        let links = all_links(&db).await;
        assert!(links.contains(&(
            format!("daily:{day}"),
            format!("project:{}", project.id),
            "planned".into()
        )));
        assert_eq!(day_start_ms(days_of(NOW)), 20_777 * 86_400_000);
    }

    #[tokio::test]
    async fn refreshing_again_replaces_the_old_index() {
        let db = Db::in_memory().await;
        let vault = Vault::new("again");
        vault.write("One.md", "[[Two]]");
        vault.write("Two.md", "x");
        refresh(&db, vault.root(), NOW).await.expect("first");
        fs::remove_file(vault.0.join("Two.md")).expect("remove");

        let stats = refresh(&db, vault.root(), NOW).await.expect("second");

        assert_eq!(stats, IndexStats { notes: 1, links: 0 });
        assert!(all_links(&db).await.is_empty());
    }
}
