/// What the graph needs from one markdown note: links, tags and headings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedNote {
    /// the note names it links to, `[[name|alias]]` and `[[name#heading]]` reduced to `name`
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub headings: Vec<String>,
}

pub fn parse(text: &str) -> ParsedNote {
    let (front_tags, body) = split_front(text);
    let mut note = ParsedNote {
        tags: front_tags,
        ..ParsedNote::default()
    };
    let mut in_code = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if let Some(heading) = heading_of(line) {
            note.headings.push(heading);
        }
        links_in(line, &mut note.links);
        tags_in(line, &mut note.tags);
    }
    dedupe(&mut note.links);
    dedupe(&mut note.tags);
    note
}

fn dedupe(items: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    items.retain(|item| seen.insert(item.to_lowercase()));
}

fn heading_of(line: &str) -> Option<String> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    let rest = line[hashes..].strip_prefix(' ')?;
    ((1..=6).contains(&hashes) && !rest.trim().is_empty()).then(|| rest.trim().to_string())
}

fn links_in(line: &str, out: &mut Vec<String>) {
    let mut rest = line;
    while let Some(open) = rest.find("[[") {
        let after = &rest[open + 2..];
        let Some(close) = after.find("]]") else { break };
        let inner = &after[..close];
        // `![[image.png]]` embeds are not notes
        let embedded = rest[..open].ends_with('!');
        let name = inner.split(['|', '#']).next().unwrap_or("").trim();
        let name = name.rsplit('/').next().unwrap_or(name).trim();
        let name = name.strip_suffix(".md").unwrap_or(name);
        // a link to a file that is not a note simply finds no target later
        if !embedded && !name.is_empty() {
            out.push(name.to_string());
        }
        rest = &after[close + 2..];
    }
}

fn tags_in(line: &str, out: &mut Vec<String>) {
    let mut previous = ' ';
    for (at, c) in line.char_indices() {
        if c == '#' && previous.is_whitespace() {
            let tag: String = line[at + 1..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '/'))
                .collect();
            if tag.chars().next().is_some_and(char::is_alphabetic) {
                out.push(tag);
            }
        }
        previous = c;
    }
}

// `tags: [a, b]`, `tags: a, b` or a dash list inside the frontmatter
fn split_front(text: &str) -> (Vec<String>, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))
    else {
        return (Vec::new(), text);
    };
    let Some(close) = rest.find("\n---") else {
        return (Vec::new(), text);
    };
    let block = &rest[..close];
    let body = &rest[close + 4..];
    let mut tags = Vec::new();
    let mut in_list = false;
    for line in block.lines() {
        if let Some(value) = line.strip_prefix("tags:") {
            in_list = value.trim().is_empty();
            tags.extend(
                value
                    .trim()
                    .trim_matches(['[', ']'])
                    .split(',')
                    .map(|t| t.trim().trim_matches(['"', '\'', '#']).to_string())
                    .filter(|t| !t.is_empty()),
            );
        } else if in_list && let Some(item) = line.trim().strip_prefix("- ") {
            tags.push(item.trim().trim_matches(['"', '\'', '#']).to_string());
        } else {
            in_list = false;
        }
    }
    (tags, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikilinks_drop_aliases_and_headings_and_paths() {
        let note =
            parse("See [[Alpha]], [[beta|the second]], [[gamma#Part 2]] and [[folder/delta]].");
        assert_eq!(note.links, ["Alpha", "beta", "gamma", "delta"]);
    }

    #[test]
    fn embeds_and_empty_links_are_ignored() {
        let note = parse("![[photo.png]] [[]] [[ ]] [[real]]");
        assert_eq!(note.links, ["real"]);
    }

    #[test]
    fn a_link_to_a_note_with_the_extension_is_kept() {
        assert_eq!(parse("[[plan.md]]").links, ["plan"]);
    }

    #[test]
    fn inline_tags_need_a_letter_and_no_heading_space() {
        let note = parse("# Title\nwork on #rust and #team/core, not C# or # 5 or #123");
        assert_eq!(note.tags, ["rust", "team/core"]);
        assert_eq!(note.headings, ["Title"]);
    }

    #[test]
    fn frontmatter_tags_come_in_three_shapes() {
        assert_eq!(parse("---\ntags: [a, b]\n---\nbody").tags, ["a", "b"]);
        assert_eq!(parse("---\ntags: a, #b\n---\nbody").tags, ["a", "b"]);
        assert_eq!(
            parse("---\ntags:\n  - a\n  - b\nother: x\n---\n").tags,
            ["a", "b"]
        );
    }

    #[test]
    fn code_blocks_are_skipped_and_duplicates_merged() {
        let note = parse("```\n[[hidden]] #nope\n```\n[[One]] [[one]] #T #t");
        assert_eq!(note.links, ["One"]);
        assert_eq!(note.tags, ["T"]);
    }

    #[test]
    fn headings_of_every_level_are_listed() {
        let note = parse("# A\n## B\n####### no\n#nospace\n### C");
        assert_eq!(note.headings, ["A", "B", "C"]);
    }
}
