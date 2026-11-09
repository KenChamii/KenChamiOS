use crate::domain::media_transfer::{MAX_ROWS, ParsedFile, RawRow};
use crate::error::{AppError, AppResult};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Reads a MyAnimeList export (`<anime>` and `<manga>` entries). The file is only parsed as
/// data: DTDs are refused and nothing in it is followed or run.
pub fn parse(bytes: &[u8]) -> AppResult<ParsedFile> {
    let text = std::str::from_utf8(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes))
        .map_err(|_| AppError::Invalid("the file is not UTF-8 text, extract it first"))?;
    let not_xml = || AppError::Invalid("the file is not a valid MyAnimeList export");
    let mut reader = Reader::from_str(text);
    let mut parsed = ParsedFile::default();
    let mut entries = 0;
    let mut entry: Option<RawRow> = None;
    let mut field: Option<String> = None;
    let mut buffer = String::new();

    loop {
        match reader.read_event().map_err(|_| not_xml())? {
            Event::DocType(_) => {
                return Err(AppError::Invalid(
                    "the file has a DTD, which is not allowed",
                ));
            }
            Event::Start(start) => {
                let name = start.local_name().as_ref().to_string();
                if entry.is_none() && (name == "anime" || name == "manga") {
                    entries += 1;
                    if entries > MAX_ROWS {
                        return Err(AppError::Invalid(
                            "the file has too many entries (the limit is 5000)",
                        ));
                    }
                    entry = Some(RawRow {
                        line: entries,
                        media_type: name,
                        ..RawRow::default()
                    });
                } else if entry.is_some() {
                    field = Some(name);
                    buffer.clear();
                }
            }
            Event::Text(text) => buffer.push_str(&text.xml10_content()),
            Event::CData(data) => buffer.push_str(&data.xml10_content()),
            Event::GeneralRef(reference) => {
                if let Ok(Some(c)) = reference.resolve_char_ref() {
                    buffer.push(c);
                } else if let Some(c) = predefined_entity(&reference.xml10_content()) {
                    buffer.push(c);
                }
            }
            Event::End(end) => {
                let name = end.local_name().as_ref().to_string();
                if field.as_deref() == Some(name.as_str()) {
                    if let (Some(row), Some(field)) = (entry.as_mut(), field.take()) {
                        assign(row, &field, buffer.trim());
                    }
                } else if (name == "anime" || name == "manga")
                    && let Some(row) = entry.take()
                {
                    parsed.push(row);
                }
            }
            Event::Eof => {
                if entry.is_some() {
                    return Err(not_xml());
                }
                break;
            }
            _ => {}
        }
    }
    if entries == 0 {
        return Err(AppError::Invalid(
            "no anime or manga entries found, is this a MyAnimeList export?",
        ));
    }
    Ok(parsed)
}

fn predefined_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

// anime and manga entries use different element names for the same things
fn assign(row: &mut RawRow, field: &str, value: &str) {
    let value = value.to_string();
    match field {
        "series_title" | "manga_title" => row.title = value,
        "series_episodes" | "manga_chapters" => {
            // MyAnimeList writes 0 when the length is unknown
            row.total = if value == "0" { String::new() } else { value };
        }
        "my_watched_episodes" | "my_read_chapters" => row.progress = value,
        "my_start_date" => row.started = value,
        "my_finish_date" => row.finished = value,
        "my_score" => row.rating = value,
        "my_status" => row.status = status_of(&value),
        _ => {}
    }
}

// status names and the numeric codes both appear in exports
fn status_of(value: &str) -> String {
    let status = match value.to_lowercase().as_str() {
        "watching" | "reading" | "1" => "in_progress",
        "completed" | "2" => "completed",
        "on-hold" | "onhold" | "on hold" | "3" => "on_hold",
        "dropped" | "4" => "dropped",
        "plan to watch" | "plan to read" | "6" => "planned",
        other => return other.to_string(),
    };
    status.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::civil_date;
    use crate::domain::media::{MediaStatus, MediaType};

    const EXPORT: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<myanimelist>
  <myinfo><user_id>1</user_id><user_name>someone</user_name></myinfo>
  <anime>
    <series_animedb_id>52991</series_animedb_id>
    <series_title><![CDATA[Sousou no Frieren]]></series_title>
    <series_type>TV</series_type>
    <series_episodes>28</series_episodes>
    <my_watched_episodes>12</my_watched_episodes>
    <my_start_date>2023-10-01</my_start_date>
    <my_finish_date>0000-00-00</my_finish_date>
    <my_score>9</my_score>
    <my_status>Watching</my_status>
  </anime>
  <anime>
    <series_title>Tom &amp; Jerry &#33;</series_title>
    <series_episodes>0</series_episodes>
    <my_watched_episodes>0</my_watched_episodes>
    <my_score>0</my_score>
    <my_status>Plan to Watch</my_status>
  </anime>
  <manga>
    <manga_title><![CDATA[Berserk]]></manga_title>
    <manga_chapters>0</manga_chapters>
    <my_read_chapters>140</my_read_chapters>
    <my_score>10</my_score>
    <my_status>2</my_status>
  </manga>
  <anime>
    <series_title></series_title>
    <my_status>Dropped</my_status>
  </anime>
</myanimelist>"#;

    #[test]
    fn entries_are_read_with_their_fields() {
        let parsed = parse(EXPORT.as_bytes()).expect("parse");

        assert_eq!(parsed.rows.len(), 3);
        let frieren = &parsed.rows[0];
        assert_eq!(frieren.media_type, MediaType::Anime);
        assert_eq!(frieren.details.title, "Sousou no Frieren");
        assert_eq!(frieren.details.progress_total, Some(28));
        assert_eq!(frieren.progress, 12);
        assert_eq!(frieren.rating, Some(9));
        assert_eq!(frieren.status, MediaStatus::InProgress);
        assert_eq!(frieren.started_at, civil_date::parse("2023-10-01"));
        assert_eq!(frieren.finished_at, None);
    }

    #[test]
    fn zero_means_unknown_total_and_no_score() {
        let parsed = parse(EXPORT.as_bytes()).expect("parse");

        let tom = &parsed.rows[1];
        assert_eq!(tom.details.title, "Tom & Jerry !");
        assert_eq!(tom.details.progress_total, None);
        assert_eq!(tom.rating, None);
        assert_eq!(tom.status, MediaStatus::Planned);
    }

    #[test]
    fn manga_entries_and_numeric_statuses_work() {
        let parsed = parse(EXPORT.as_bytes()).expect("parse");

        let berserk = &parsed.rows[2];
        assert_eq!(berserk.media_type, MediaType::Manga);
        assert_eq!(berserk.progress, 140);
        assert_eq!(berserk.status, MediaStatus::Completed);
    }

    #[test]
    fn broken_entries_are_reported_by_their_number() {
        let parsed = parse(EXPORT.as_bytes()).expect("parse");

        assert_eq!(parsed.rejected.len(), 1);
        assert_eq!(parsed.rejected[0].line, 4);
        assert_eq!(parsed.rejected[0].reason, "the title is empty");
    }

    #[test]
    fn a_dtd_is_refused() {
        let xml = "<?xml version=\"1.0\"?><!DOCTYPE x [<!ENTITY a \"b\">]><myanimelist/>";

        assert!(parse(xml.as_bytes()).is_err());
    }

    #[test]
    fn other_files_are_refused() {
        assert!(parse(b"<html><body>hi</body></html>").is_err());
        assert!(parse(b"not xml at all <<<").is_err());
        assert!(parse(b"\xFF\xFE").is_err());
        assert!(parse(b"<myanimelist><anime><series_title>x").is_err());
    }
}
