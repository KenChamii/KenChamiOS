use super::dates::parse_day;
use super::file::{self, Region};
use super::specs::{self, APP_DIR, Spec, Target};
use crate::db::vault::{self, FileState};
use crate::db::{Db, daily_notes, docs, tasks};
use crate::domain::project::INBOX_ID;
use crate::domain::task::{Priority, TaskCreate};
use crate::domain::{DocKind, TaskStatus};
use crate::error::{AppError, AppResult};
use crate::sync::codes::sha256_hex;
use crate::workflow::safe_path::resolve_inside;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const IMPORTED_MARK: &str = "<!-- kco:imported -->";
const INBOX_FILE: &str = "Inbox.md";
const INBOX_HELP: &str =
    "# Inbox\n\nWrite one item per line as a list item. The app turns each into an inbox task.\n\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Added,
    Updated,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    ToVault,
    ToApp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub path: String,
    pub kind: Kind,
    pub direction: Direction,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub changes: Vec<Change>,
    /// files that were already in step
    pub skipped: usize,
    /// new Inbox.md lines turned into tasks (or that would be)
    pub inbox_imported: usize,
}

enum Decision {
    Unchanged,
    Write { added: bool },
    Import { conflict: bool },
}

fn hash_regions(regions: &[Region]) -> String {
    let joined: String = regions
        .iter()
        .map(|r| format!("{}\u{0}{}\u{0}", r.name, r.content))
        .collect();
    sha256_hex(joined.as_bytes())
}

fn decide(spec: &Spec, existing: Option<&str>, state: Option<&FileState>) -> Decision {
    let Some(text) = existing else {
        return Decision::Write { added: true };
    };
    let app_hash = hash_regions(&spec.regions);
    let two_way = spec.target.two_way();
    let Some(state) = state else {
        // a file that was never synced and already says the same needs nothing
        let same =
            file::refresh(text, &spec.front_with(text), &spec.regions).is_ok_and(|t| t == text);
        return match (same, two_way) {
            (true, _) => Decision::Unchanged,
            (false, true) => Decision::Import { conflict: false },
            (false, false) => Decision::Write { added: false },
        };
    };
    let vault_changed = sha256_hex(text.as_bytes()) != state.hash;
    let app_changed = app_hash != state.app_hash;
    match (vault_changed, app_changed, two_way) {
        (false, false, _) => Decision::Unchanged,
        (false, true, _) | (true, _, false) => Decision::Write { added: false },
        (true, false, true) => Decision::Import { conflict: false },
        (true, true, true) => Decision::Import { conflict: true },
    }
}

impl Spec {
    // the frontmatter as the file has it, so comparing does not see a new timestamp
    fn front_with(&self, text: &str) -> file::Front {
        let updated = file::split(text).0.map_or(0, |front| front.updated);
        file::Front {
            updated,
            ..self.front.clone()
        }
    }

    fn content_for(&self, existing: Option<&str>, now: i64) -> AppResult<String> {
        let Some(text) = existing else {
            return Ok(file::fresh(
                &file::Front {
                    updated: now,
                    ..self.front.clone()
                },
                &self.title,
                &self.regions,
            ));
        };
        let same = file::refresh(text, &self.front_with(text), &self.regions)?;
        if same == text {
            return Ok(same);
        }
        file::refresh(
            text,
            &file::Front {
                updated: now,
                ..self.front.clone()
            },
            &self.regions,
        )
    }
}

/// Plans or runs one pass over the vault: app files out, vault edits in. With `apply` false
/// nothing is written anywhere.
pub async fn run(db: &Db, data_dir: &Path, root: &str, now: i64, apply: bool) -> AppResult<Report> {
    if !Path::new(root.trim()).is_dir() {
        return Err(AppError::Invalid("the vault folder does not exist"));
    }
    let found = scan(root)?;
    let extra_days: Vec<String> = found.daily_days.clone();
    let mut report = Report::default();

    for spec in specs::all(db, now, &extra_days).await? {
        // a renamed or moved file is still found by its kco_id
        let path = found
            .by_id
            .get(&spec.front.kco_id)
            .cloned()
            .unwrap_or_else(|| spec.path.clone());
        if apply && path != spec.path {
            vault::forget(db, &spec.path).await?;
        }
        process(db, data_dir, root, &spec, &path, now, apply, &mut report).await?;
    }
    report.inbox_imported = inbox(db, root, apply, &mut report).await?;
    if apply {
        super::guides::write_missing(root)?;
        vault::set_last_sync(db, now).await?;
    }
    Ok(report)
}

struct Found {
    by_id: HashMap<String, String>,
    daily_days: Vec<String>,
}

// every markdown file the app owns, by the id in its frontmatter
fn scan(root: &str) -> AppResult<Found> {
    let mut found = Found {
        by_id: HashMap::new(),
        daily_days: Vec::new(),
    };
    let base = PathBuf::from(root.trim());
    let mut folders = vec![base.join(APP_DIR)];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                let relative = path
                    .strip_prefix(&base)
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                let text = fs::read_to_string(&path).unwrap_or_default();
                if let (Some(front), _) = file::split(&text) {
                    found.by_id.insert(front.kco_id, relative.clone());
                }
                let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
                let in_daily = relative.starts_with(&format!("{APP_DIR}/Daily/"));
                if let (true, Some(stem)) = (in_daily, stem)
                    && parse_day(&stem).is_some()
                {
                    found.daily_days.push(stem);
                }
            }
        }
    }
    Ok(found)
}

#[allow(clippy::too_many_arguments)]
async fn process(
    db: &Db,
    data_dir: &Path,
    root: &str,
    spec: &Spec,
    path: &str,
    now: i64,
    apply: bool,
    report: &mut Report,
) -> AppResult<()> {
    let abs = resolve_inside(root, path)?;
    let existing = fs::read_to_string(&abs).ok();
    let state = vault::get(db, path).await?;
    match decide(spec, existing.as_deref(), state.as_ref()) {
        Decision::Unchanged => {
            report.skipped += 1;
            if apply && state.is_none() {
                record(db, path, spec, &abs, now).await?;
            }
        }
        Decision::Write { added } => {
            let content = spec.content_for(existing.as_deref(), now)?;
            if existing.as_deref() == Some(content.as_str()) {
                report.skipped += 1;
            } else {
                report.changes.push(Change {
                    path: path.to_string(),
                    kind: if added { Kind::Added } else { Kind::Updated },
                    direction: Direction::ToVault,
                });
                if apply {
                    if let Some(dir) = abs.parent() {
                        fs::create_dir_all(dir)?;
                    }
                    fs::write(&abs, &content)?;
                }
            }
            if apply {
                record(db, path, spec, &abs, now).await?;
            }
        }
        Decision::Import { conflict } => {
            report.changes.push(Change {
                path: path.to_string(),
                kind: if conflict {
                    Kind::Conflict
                } else {
                    Kind::Updated
                },
                direction: Direction::ToApp,
            });
            if apply {
                let text = existing.unwrap_or_default();
                if conflict {
                    keep_app_version(data_dir, spec, now)?;
                }
                import(db, spec, &text).await?;
                rewrite_if_adopted(db, spec, &abs, &text, now).await?;
                record(db, path, spec, &abs, now).await?;
            }
        }
    }
    Ok(())
}

// the file as it is now and the app as it is now are what the next pass compares against
async fn record(db: &Db, path: &str, spec: &Spec, abs: &Path, now: i64) -> AppResult<()> {
    let rebuilt = rebuild(db, spec).await?;
    let text = fs::read_to_string(abs).unwrap_or_default();
    let mtime = fs::metadata(abs)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0));
    vault::put(
        db,
        &FileState {
            path: path.to_string(),
            kco_id: spec.front.kco_id.clone(),
            hash: sha256_hex(text.as_bytes()),
            app_hash: hash_regions(&rebuilt.regions),
            mtime,
            synced_at: now,
        },
    )
    .await
}

async fn rebuild(db: &Db, spec: &Spec) -> AppResult<Spec> {
    match &spec.target {
        Target::Daily { day } => specs::daily(db, day).await,
        Target::Project { id } => specs::project_specs(db)
            .await?
            .into_iter()
            .find(|s| s.target == Target::Project { id: id.clone() })
            .ok_or(AppError::NotFound("project")),
        _ => Ok(spec.clone()),
    }
}

// the app's side, kept before the vault overwrites it, so a conflict never loses text
fn keep_app_version(data_dir: &Path, spec: &Spec, now: i64) -> AppResult<()> {
    let dir = data_dir.join("vault-conflicts");
    fs::create_dir_all(&dir)?;
    let name = spec.path.replace('/', "-");
    let body: String = spec
        .regions
        .iter()
        .map(|r| format!("## {}\n\n{}\n\n", r.name, r.content))
        .collect();
    fs::write(dir.join(format!("{now}-{name}")), body)?;
    Ok(())
}

async fn import(db: &Db, spec: &Spec, text: &str) -> AppResult<()> {
    let (_, body) = file::split(text);
    let marked = |name| file::region(body, name);
    match &spec.target {
        Target::Daily { day } => {
            // a plain note has no markers yet; its whole text is the note
            let note = marked("note").unwrap_or(body.trim());
            let current = spec
                .regions
                .first()
                .map(|r| r.content.as_str())
                .unwrap_or("");
            if note.trim() != current {
                daily_notes::save(db, day, note.trim()).await?;
            }
            if let Some(tasks_text) = marked("tasks") {
                import_ticks(db, spec, tasks_text).await?;
            }
        }
        Target::Project { id } => {
            for (name, kind, title) in [
                ("idea", DocKind::Idea, "Idea"),
                ("plan", DocKind::Plan, "Plan"),
            ] {
                let Some(content) = marked(name) else {
                    continue;
                };
                let current = spec
                    .regions
                    .iter()
                    .find(|r| r.name == name)
                    .map_or("", |r| r.content.as_str());
                if content.trim() != current {
                    docs::upsert(db, id, kind, title, content.trim()).await?;
                }
            }
        }
        Target::Dashboard | Target::Review => {}
    }
    Ok(())
}

// `- [x] title <!-- kco:task:ID -->`; only tasks this file listed can change
async fn import_ticks(db: &Db, spec: &Spec, region: &str) -> AppResult<()> {
    for line in region.lines() {
        let Some((id, checked)) = parse_tick(line) else {
            continue;
        };
        let Some((_, was_done)) = spec.tick_tasks.iter().find(|(task, _)| *task == id) else {
            continue;
        };
        if checked != *was_done {
            let status = if checked {
                TaskStatus::Done
            } else {
                TaskStatus::Todo
            };
            tasks::move_to(db, &id, status, None, None).await?;
        }
    }
    Ok(())
}

pub fn parse_tick(line: &str) -> Option<(String, bool)> {
    let line = line.trim_start();
    let checked = if line.starts_with("- [x]") || line.starts_with("- [X]") {
        true
    } else if line.starts_with("- [ ]") {
        false
    } else {
        return None;
    };
    let marker = "<!-- kco:task:";
    let from = line.find(marker)? + marker.len();
    let to = from + line[from..].find(" -->")?;
    Some((line[from..to].to_string(), checked))
}

// a note written by hand gets the frontmatter and markers, its text moving into the region
async fn rewrite_if_adopted(
    db: &Db,
    spec: &Spec,
    abs: &Path,
    text: &str,
    now: i64,
) -> AppResult<()> {
    let (front, body) = file::split(text);
    let marker_name = spec.regions.first().map_or("", |r| r.name);
    if front.is_some() || file::region(body, marker_name).is_some() {
        return Ok(());
    }
    let rebuilt = rebuild(db, spec).await?;
    let content = rebuilt.content_for(None, now)?;
    fs::write(abs, content)?;
    Ok(())
}

async fn inbox(db: &Db, root: &str, apply: bool, report: &mut Report) -> AppResult<usize> {
    let path = format!("{APP_DIR}/{INBOX_FILE}");
    let abs = resolve_inside(root, &path)?;
    let Ok(text) = fs::read_to_string(&abs) else {
        if apply {
            if let Some(dir) = abs.parent() {
                fs::create_dir_all(dir)?;
            }
            let front = file::Front {
                kco_id: "inbox".into(),
                kco_type: "inbox".into(),
                updated: 0,
            };
            fs::write(&abs, file::assemble(&front, INBOX_HELP))?;
        }
        report.changes.push(Change {
            path,
            kind: Kind::Added,
            direction: Direction::ToVault,
        });
        return Ok(0);
    };
    let mut imported = 0;
    let mut out = Vec::new();
    for line in text.lines() {
        match inbox_item(line) {
            Some(title) => {
                imported += 1;
                if apply {
                    tasks::create(db, inbox_task(&title)).await?;
                }
                out.push(format!("- [x] {title} {IMPORTED_MARK}"));
            }
            None => out.push(line.to_string()),
        }
    }
    if imported > 0 {
        report.changes.push(Change {
            path,
            kind: Kind::Updated,
            direction: Direction::ToApp,
        });
        if apply {
            let mut rewritten = out.join("\n");
            if text.ends_with('\n') {
                rewritten.push('\n');
            }
            fs::write(&abs, rewritten)?;
        }
    }
    Ok(imported)
}

fn inbox_task(title: &str) -> TaskCreate {
    TaskCreate {
        project_id: INBOX_ID.to_string(),
        parent_id: None,
        title: title.to_string(),
        body_md: String::new(),
        acceptance_md: String::new(),
        related_files: Vec::new(),
        priority: Priority::Medium,
        due_at: None,
        tags: Vec::new(),
        recurrence: None,
    }
}

/// The task title of a new list line, or `None` for any other line and for imported ones.
pub fn inbox_item(line: &str) -> Option<String> {
    let line = line.trim();
    if line.contains(IMPORTED_MARK) {
        return None;
    }
    let item = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))?;
    let item = item.strip_prefix("[ ] ").unwrap_or(item);
    if item.starts_with("[x] ") || item.starts_with("[X] ") {
        return None;
    }
    let title = item.trim();
    (!title.is_empty()).then(|| title.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_are_read_with_their_task_id() {
        assert_eq!(
            parse_tick("- [x] Write <!-- kco:task:abc -->"),
            Some(("abc".into(), true))
        );
        assert_eq!(
            parse_tick("  - [ ] Write <!-- kco:task:abc -->"),
            Some(("abc".into(), false))
        );
        assert_eq!(parse_tick("- [x] no id"), None);
        assert_eq!(parse_tick("plain line"), None);
    }

    #[test]
    fn only_new_list_lines_are_inbox_items() {
        assert_eq!(inbox_item("- buy milk").as_deref(), Some("buy milk"));
        assert_eq!(inbox_item("* call mum").as_deref(), Some("call mum"));
        assert_eq!(inbox_item("- [ ] open box").as_deref(), Some("open box"));
        assert_eq!(inbox_item("- [x] done"), None);
        assert_eq!(inbox_item("- x <!-- kco:imported -->"), None);
        assert_eq!(inbox_item("# Inbox"), None);
        assert_eq!(inbox_item("-"), None);
    }
}
