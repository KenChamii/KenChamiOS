use crate::error::{AppError, AppResult};

const FENCE: &str = "---";

/// The three flat keys every app file carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Front {
    pub kco_id: String,
    pub kco_type: String,
    pub updated: i64,
}

/// One generated block of a file. Text around it belongs to the user and is never touched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub name: &'static str,
    pub heading: &'static str,
    pub content: String,
}

fn start(name: &str) -> String {
    format!("<!-- kco:start {name} -->")
}

fn end(name: &str) -> String {
    format!("<!-- kco:end {name} -->")
}

/// The frontmatter, when the file has one with a `kco_id`, and the text after it.
pub fn split(text: &str) -> (Option<Front>, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))
    else {
        return (None, text);
    };
    let Some(close) = rest.find("\n---") else {
        return (None, text);
    };
    let block = &rest[..close];
    let after = &rest[close + 4..];
    // the blank line the app puts after the frontmatter is not part of the body
    let body = after.trim_start_matches(['\r', '\n']);
    (parse_front(block), body)
}

fn parse_front(block: &str) -> Option<Front> {
    let value = |key: &str| {
        block.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name.trim() == key).then(|| value.trim().trim_matches('"').to_string())
        })
    };
    Some(Front {
        kco_id: value("kco_id").filter(|id| !id.is_empty())?,
        kco_type: value("kco_type")?,
        updated: value("updated").and_then(|v| v.parse().ok()).unwrap_or(0),
    })
}

fn front_text(front: &Front) -> String {
    format!(
        "{FENCE}\nkco_id: {}\nkco_type: {}\nupdated: {}\n{FENCE}\n",
        front.kco_id, front.kco_type, front.updated
    )
}

/// What sits between the markers of `name`, without the lines the markers are on.
pub fn region<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    let open = start(name);
    let from = body.find(&open)? + open.len();
    let to = from + body[from..].find(&end(name))?;
    Some(body[from..to].trim_matches(['\r', '\n']))
}

/// `body` with the content of each region replaced; regions the file lacks are added at the end.
/// Everything outside the markers stays as it was.
pub fn with_regions(body: &str, regions: &[Region]) -> String {
    let mut out = body.to_string();
    for item in regions {
        let open = start(item.name);
        let close = end(item.name);
        let found = out.find(&open).and_then(|at| {
            let from = at + open.len();
            out[from..].find(&close).map(|len| (from, from + len))
        });
        match found {
            Some((from, to)) => out.replace_range(from..to, &format!("\n{}\n", item.content)),
            None => {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                let heading = if item.heading.is_empty() {
                    String::new()
                } else {
                    format!("\n## {}\n", item.heading)
                };
                out.push_str(&format!("{heading}{open}\n{}\n{close}\n", item.content));
            }
        }
    }
    out
}

/// The whole file: frontmatter, then the body.
pub fn assemble(front: &Front, body: &str) -> String {
    format!("{}\n{body}", front_text(front))
}

/// A new file: a title and every region.
pub fn fresh(front: &Front, title: &str, regions: &[Region]) -> String {
    assemble(front, &with_regions(&format!("# {title}\n"), regions))
}

/// The file with its generated regions refreshed; the frontmatter id must be the expected one.
pub fn refresh(existing: &str, front: &Front, regions: &[Region]) -> AppResult<String> {
    let (found, body) = split(existing);
    if let Some(found) = found
        && found.kco_id != front.kco_id
    {
        return Err(AppError::Invalid(
            "the vault file belongs to another record",
        ));
    }
    Ok(assemble(front, &with_regions(body, regions)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn front() -> Front {
        Front {
            kco_id: "abc".into(),
            kco_type: "daily".into(),
            updated: 5,
        }
    }

    fn note(content: &str) -> Region {
        Region {
            name: "note",
            heading: "Note",
            content: content.into(),
        }
    }

    #[test]
    fn a_new_file_has_frontmatter_a_title_and_marked_regions() {
        let text = fresh(&front(), "2026-11-16", &[note("hello")]);
        assert!(text.starts_with("---\nkco_id: abc\nkco_type: daily\nupdated: 5\n---\n"));
        assert!(text.contains("# 2026-11-16"));
        assert!(text.contains("<!-- kco:start note -->\nhello\n<!-- kco:end note -->"));
    }

    #[test]
    fn the_frontmatter_and_region_round_trip() {
        let text = fresh(&front(), "day", &[note("one\n\ntwo")]);
        let (found, body) = split(&text);
        assert_eq!(found, Some(front()));
        assert_eq!(region(body, "note"), Some("one\n\ntwo"));
        assert_eq!(region(body, "other"), None);
    }

    #[test]
    fn text_outside_the_markers_is_kept() {
        let text = fresh(&front(), "day", &[note("old")]);
        let typed = text.replace("# day", "# day\n\nmy own words");
        let again = refresh(&typed, &front(), &[note("new")]).expect("refresh");
        assert!(again.contains("my own words"));
        assert!(again.contains("\nnew\n"));
        assert!(!again.contains("old"));
    }

    #[test]
    fn a_missing_region_is_added_at_the_end() {
        let body = with_regions("# t\nfree text", &[note("x")]);
        assert!(body.starts_with("# t\nfree text\n"));
        assert_eq!(region(&body, "note"), Some("x"));
    }

    #[test]
    fn refreshing_twice_changes_nothing() {
        let first = fresh(&front(), "day", &[note("same")]);
        assert_eq!(
            refresh(&first, &front(), &[note("same")]).expect("refresh"),
            first
        );
    }

    #[test]
    fn a_file_of_another_record_is_refused() {
        let other = Front {
            kco_id: "zzz".into(),
            ..front()
        };
        let text = fresh(&other, "day", &[note("x")]);
        assert!(refresh(&text, &front(), &[note("y")]).is_err());
    }

    #[test]
    fn windows_line_endings_and_a_bom_are_read() {
        let text = "\u{feff}---\r\nkco_id: abc\r\nkco_type: daily\r\nupdated: 1\r\n---\r\n# t\r\n";
        let (found, body) = split(text);
        assert_eq!(found.map(|f| f.kco_id), Some("abc".to_string()));
        assert!(body.starts_with("# t"));
    }

    #[test]
    fn a_plain_note_has_no_frontmatter() {
        let (found, body) = split("# just a note\n");
        assert_eq!(found, None);
        assert_eq!(body, "# just a note\n");
    }
}
