/// Turns provider descriptions into plain text: line breaks kept, other tags and entities dropped.
pub fn plain_text(source: &str) -> String {
    let mut text = String::with_capacity(source.len());
    let mut tag = String::new();
    let mut inside = false;
    for c in source.chars() {
        match c {
            '<' => {
                inside = true;
                tag.clear();
            }
            '>' if inside => {
                inside = false;
                if is_break(&tag) {
                    text.push('\n');
                }
            }
            _ if inside => tag.push(c),
            _ => text.push(c),
        }
    }
    decode_entities(&text)
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn is_break(tag: &str) -> bool {
    let name = tag.trim_start_matches('/').trim_end_matches('/').trim();
    name.eq_ignore_ascii_case("br") || name.eq_ignore_ascii_case("p")
}

fn decode_entities(text: &str) -> String {
    // &amp; goes last so "&amp;lt;" does not turn into "<"
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_become_new_lines_and_other_tags_vanish() {
        assert_eq!(
            plain_text("One<br>Two<br />Three <i>x</i>"),
            "One\nTwo\nThree x"
        );
        assert_eq!(plain_text("<p>Para</p><p>Next</p>"), "Para\n\nNext");
    }

    #[test]
    fn entities_are_decoded_once() {
        assert_eq!(
            plain_text("Tom &amp; Jerry &quot;hi&quot;"),
            "Tom & Jerry \"hi\""
        );
        assert_eq!(plain_text("&amp;lt;"), "&lt;");
    }

    #[test]
    fn plain_text_is_trimmed_and_unchanged() {
        assert_eq!(plain_text("  just text \n"), "just text");
        assert_eq!(plain_text(""), "");
    }
}
