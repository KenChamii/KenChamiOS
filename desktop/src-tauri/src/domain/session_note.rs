use super::Task;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// below this share of shared words an item is not suggested for a task
const MATCH_THRESHOLD: f64 = 0.5;

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct ParsedNote {
    /// Everything that is not part of a recognised section or checkbox list.
    pub note: String,
    pub done: Vec<String>,
    pub next: Vec<String>,
    pub changed_files: Vec<String>,
}

/// What the user confirmed in the session note preview.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfirm {
    pub project_id: String,
    pub note: String,
    /// the text as pasted
    pub summary_md: String,
    pub done: Vec<String>,
    pub next: Vec<String>,
    pub changed_files: Vec<String>,
    pub done_task_ids: Vec<String>,
    pub new_task_titles: Vec<String>,
}

#[derive(Clone, Copy)]
enum Section {
    Done,
    Next,
    Changed,
}

/// Splits a pasted session summary into done / next / changed files and the rest.
pub fn parse(text: &str) -> ParsedNote {
    let mut parsed = ParsedNote::default();
    let mut rest: Vec<&str> = Vec::new();
    let mut section: Option<Section> = None;

    for line in text.lines() {
        if let Some(item) = list_item(line) {
            match (section, item.checked) {
                (Some(Section::Done), _) | (None, Some(true)) => parsed.done.push(item.text),
                (Some(Section::Next), _) | (None, Some(false)) => parsed.next.push(item.text),
                (Some(Section::Changed), _) => {
                    parsed
                        .changed_files
                        .push(item.text.trim_matches('`').to_string());
                }
                (None, None) => rest.push(line),
            }
            continue;
        }
        if let Some(title) = heading_title(line) {
            section = section_of(&title);
            if section.is_none() {
                rest.push(line);
            }
            continue;
        }
        // blank lines only separate items inside a section
        if section.is_none() || !line.trim().is_empty() {
            rest.push(line);
        }
    }
    parsed.note = collapse_blank_lines(&rest);
    parsed
}

/// The open task whose title shares the most words with `item`, if close enough.
pub fn best_match<'a>(item: &str, tasks: &'a [Task]) -> Option<&'a Task> {
    let wanted = words(item);
    tasks
        .iter()
        .map(|task| (task, similarity(&wanted, &words(&task.title))))
        .filter(|(_, score)| *score >= MATCH_THRESHOLD)
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(task, _)| task)
}

fn similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / union as f64
}

fn words(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

struct ListItem {
    checked: Option<bool>,
    text: String,
}

fn list_item(line: &str) -> Option<ListItem> {
    let line = line.trim();
    let after_marker = ["- ", "* ", "+ "]
        .iter()
        .find_map(|marker| line.strip_prefix(marker))
        .or_else(|| numbered_item(line))?
        .trim();
    let (checked, text) = if let Some(text) = after_marker.strip_prefix("[ ]") {
        (Some(false), text)
    } else if let Some(text) = after_marker
        .strip_prefix("[x]")
        .or_else(|| after_marker.strip_prefix("[X]"))
    {
        (Some(true), text)
    } else {
        (None, after_marker)
    };
    let text = text.trim();
    (!text.is_empty()).then(|| ListItem {
        checked,
        text: text.to_string(),
    })
}

fn numbered_item(line: &str) -> Option<&str> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let rest = &line[digits..];
    rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") "))
}

/// Lowercase title of a markdown heading, a bold line, or a line like "Done:".
fn heading_title(line: &str) -> Option<String> {
    let line = line.trim();
    let title = if line.starts_with('#') {
        line.trim_start_matches('#')
    } else if let Some(bold) = line.strip_prefix("**").and_then(|l| l.strip_suffix("**")) {
        bold
    } else {
        line.strip_suffix(':')?
    };
    let title = title.trim().trim_end_matches(':').trim_matches('*').trim();
    (!title.is_empty()).then(|| title.to_lowercase())
}

fn section_of(title: &str) -> Option<Section> {
    match title {
        "done" | "completed" | "finished" | "what was done" | "accomplished" => Some(Section::Done),
        "next" | "next steps" | "to do" | "todo" | "up next" | "remaining" => Some(Section::Next),
        "changed files" | "files changed" | "modified files" | "files" => Some(Section::Changed),
        _ => None,
    }
}

fn collapse_blank_lines(lines: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in lines {
        let blank = line.trim().is_empty();
        if blank && out.last().is_none_or(|last| last.trim().is_empty()) {
            continue;
        }
        out.push(line.trim_end());
    }
    out.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Priority, TaskStatus};
    use sqlx::types::Json;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn parsed(note: &str, done: &[&str], next: &[&str], files: &[&str]) -> ParsedNote {
        ParsedNote {
            note: note.to_string(),
            done: strings(done),
            next: strings(next),
            changed_files: strings(files),
        }
    }

    #[test]
    fn parses_known_shapes() {
        let cases = [
            (
                "markdown headings",
                "Good session.\n\n## Done\n- login form\n- tests\n\n## Next\n- deploy\n\n## Changed files\n- `src/a.ts`\n- src/b.ts\n",
                parsed(
                    "Good session.",
                    &["login form", "tests"],
                    &["deploy"],
                    &["src/a.ts", "src/b.ts"],
                ),
            ),
            (
                "bold and colon headings",
                "**Done**\n* one\n\nNext steps:\n1. two\n2) three\n",
                parsed("", &["one"], &["two", "three"], &[]),
            ),
            (
                "checkbox lists outside sections",
                "Worked on auth.\n- [x] add session store\n- [ ] add logout\n- plain bullet\n",
                parsed(
                    "Worked on auth.\n- plain bullet",
                    &["add session store"],
                    &["add logout"],
                    &[],
                ),
            ),
            (
                "unknown heading ends a section and stays in the note",
                "## Done\n- a\n\n## Notes\nSome thoughts.\n- b\n",
                parsed("## Notes\nSome thoughts.\n- b", &["a"], &[], &[]),
            ),
            (
                "no structure at all",
                "Just a paragraph.\n\n\nAnd another.",
                parsed("Just a paragraph.\n\nAnd another.", &[], &[], &[]),
            ),
            (
                "case and checkbox inside a section",
                "## COMPLETED\n- [x] shipped\n",
                parsed("", &["shipped"], &[], &[]),
            ),
            ("empty input", "  \n", ParsedNote::default()),
        ];
        for (name, text, expected) in cases {
            assert_eq!(parse(text), expected, "{name}");
        }
    }

    fn task(title: &str) -> Task {
        Task {
            id: title.to_string(),
            created_at: 0,
            project_id: "p".into(),
            parent_id: None,
            title: title.to_string(),
            body_md: String::new(),
            acceptance_md: String::new(),
            related_files: Json(vec![]),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            due_at: None,
            order_key: "a".into(),
            tags: Json(vec![]),
        }
    }

    #[test]
    fn matches_close_titles_only() {
        let tasks = [
            task("Login form"),
            task("Export to markdown"),
            task("Dark mode"),
        ];
        let cases = [
            ("Added login form", Some("Login form")),
            ("export to Markdown!", Some("Export to markdown")),
            ("Fixed the sidebar", None),
            ("Implement login", None),
            ("", None),
        ];
        for (item, expected) in cases {
            let found = best_match(item, &tasks).map(|t| t.title.as_str());
            assert_eq!(found, expected, "{item}");
        }
    }
}
