use crate::domain::civil_date;
use crate::domain::media::LibraryEntry;
use crate::domain::media_transfer::{MAX_ROWS, ParsedFile, RawRow};
use crate::error::{AppError, AppResult};
use csv::StringRecord;

const COLUMNS: [&str; 16] = [
    "type",
    "title",
    "year",
    "status",
    "progress",
    "total",
    "rating",
    "genres",
    "tags",
    "notes",
    "started",
    "finished",
    "platform",
    "source",
    "external_id",
    "cover_url",
];

/// The library as CSV text; cells that a spreadsheet would run as a formula are defused.
pub fn export(entries: &[LibraryEntry]) -> AppResult<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(COLUMNS).map_err(csv_error)?;
    for entry in entries {
        let cells = [
            entry.media_type.as_str().to_string(),
            entry.title.clone(),
            entry.year.map(|y| y.to_string()).unwrap_or_default(),
            entry.status.as_str().to_string(),
            entry.progress.to_string(),
            entry
                .progress_total
                .map(|t| t.to_string())
                .unwrap_or_default(),
            entry.rating.map(|r| r.to_string()).unwrap_or_default(),
            entry.genres.join(";"),
            entry.tags.join(";"),
            entry.notes.clone(),
            entry.started_at.map(civil_date::format).unwrap_or_default(),
            entry
                .finished_at
                .map(civil_date::format)
                .unwrap_or_default(),
            entry.platform.clone().unwrap_or_default(),
            entry.source.as_str().to_string(),
            entry.external_id.clone().unwrap_or_default(),
            entry.cover_url.clone().unwrap_or_default(),
        ];
        writer
            .write_record(cells.iter().map(|cell| defuse(cell)))
            .map_err(csv_error)?;
    }
    let bytes = writer.into_inner().map_err(|_| csv_failure())?;
    String::from_utf8(bytes).map_err(|_| csv_failure())
}

/// Reads CSV text; a row that fails its checks is reported, never skipped silently.
pub fn parse(bytes: &[u8]) -> AppResult<ParsedFile> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(bytes);
    let headers: Vec<String> = reader
        .headers()
        .map_err(|_| AppError::Invalid("the file is not a readable CSV"))?
        .iter()
        .map(|header| header.trim().to_lowercase())
        .collect();
    for required in ["type", "title", "status"] {
        if !headers.iter().any(|header| header == required) {
            return Err(AppError::Invalid(
                "the CSV needs type, title and status columns",
            ));
        }
    }

    let mut parsed = ParsedFile::default();
    for (index, record) in reader.records().enumerate() {
        if index >= MAX_ROWS {
            return Err(AppError::Invalid(
                "the file has too many rows (the limit is 5000)",
            ));
        }
        // the header is line 1
        let line = index + 2;
        match record {
            Ok(record) => parsed.push(raw_of(&headers, &record, line)),
            Err(_) => parsed.reject(line, "the line could not be read"),
        }
    }
    Ok(parsed)
}

fn raw_of(headers: &[String], record: &StringRecord, line: usize) -> RawRow {
    let cell = |name: &str| {
        headers
            .iter()
            .position(|header| header == name)
            .and_then(|position| record.get(position))
            .map(restore)
            .unwrap_or_default()
    };
    RawRow {
        line,
        media_type: cell("type"),
        title: cell("title"),
        year: cell("year"),
        status: cell("status"),
        progress: cell("progress"),
        total: cell("total"),
        rating: cell("rating"),
        genres: cell("genres"),
        tags: cell("tags"),
        notes: cell("notes"),
        started: cell("started"),
        finished: cell("finished"),
        platform: cell("platform"),
        source: cell("source"),
        external_id: cell("external_id"),
        cover_url: cell("cover_url"),
    }
}

const FORMULA_STARTS: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// A leading apostrophe makes spreadsheets read the cell as text instead of a formula.
fn defuse(cell: &str) -> String {
    if cell.starts_with(FORMULA_STARTS) {
        format!("'{cell}")
    } else {
        cell.to_string()
    }
}

/// Undoes `defuse`, so an exported file imports back unchanged.
fn restore(cell: &str) -> String {
    match cell.strip_prefix('\'') {
        Some(rest) if rest.starts_with(FORMULA_STARTS) => rest.to_string(),
        _ => cell.to_string(),
    }
}

fn csv_error(_: csv::Error) -> AppError {
    csv_failure()
}

fn csv_failure() -> AppError {
    AppError::Invalid("the library could not be written as CSV")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::media::{MediaSource, MediaStatus, MediaType};
    use sqlx::types::Json;

    fn entry(title: &str) -> LibraryEntry {
        LibraryEntry {
            id: "e".into(),
            created_at: 0,
            updated_at: 0,
            media_item_id: "i".into(),
            media_type: MediaType::Manga,
            title: title.into(),
            cover_url: Some("https://x.test/c.jpg".into()),
            cover_cache_path: None,
            cover_path: None,
            year: Some(1989),
            genres: Json(vec!["Dark Fantasy".into(), "Seinen".into()]),
            synopsis: String::new(),
            source: MediaSource::Anilist,
            external_id: Some("30002".into()),
            status: MediaStatus::InProgress,
            progress: 140,
            progress_total: None,
            rating: Some(10),
            notes: "line one\nline \"two\", with comma".into(),
            tags: Json(vec!["classic".into()]),
            started_at: civil_date::parse("2024-03-01"),
            finished_at: None,
            platform: None,
        }
    }

    #[test]
    fn an_export_imports_back_unchanged() {
        let text = export(&[entry("Berserk")]).expect("export");

        let parsed = parse(text.as_bytes()).expect("parse");

        assert!(parsed.rejected.is_empty(), "{:?}", parsed.rejected);
        let row = &parsed.rows[0];
        assert_eq!(row.media_type, MediaType::Manga);
        assert_eq!(row.details.title, "Berserk");
        assert_eq!(row.details.genres, ["Dark Fantasy", "Seinen"]);
        assert_eq!(row.progress, 140);
        assert_eq!(row.rating, Some(10));
        assert_eq!(row.notes, "line one\nline \"two\", with comma");
        assert_eq!(row.source, MediaSource::Anilist);
        assert_eq!(row.external_id.as_deref(), Some("30002"));
        assert_eq!(row.started_at, civil_date::parse("2024-03-01"));
    }

    #[test]
    fn formula_cells_are_defused_and_restored() {
        let mut dangerous = entry("=HYPERLINK(\"http://evil\")");
        dangerous.notes = "+1 and @home".into();
        dangerous.platform = Some("-cmd".into());

        let text = export(&[dangerous]).expect("export");

        assert!(text.contains("'=HYPERLINK"), "{text}");
        assert!(text.contains("'+1 and @home"), "{text}");
        for line in text.lines().skip(1) {
            assert!(!line.starts_with(['=', '+', '-', '@']), "{line}");
        }
        let row = &parse(text.as_bytes()).expect("parse").rows[0];
        assert_eq!(row.details.title, "=HYPERLINK(\"http://evil\")");
        assert_eq!(row.notes, "+1 and @home");
        assert_eq!(row.platform.as_deref(), Some("-cmd"));
    }

    #[test]
    fn an_ordinary_leading_apostrophe_is_kept() {
        assert_eq!(restore("'Til Death"), "'Til Death");
        assert_eq!(defuse("Plain"), "Plain");
    }

    #[test]
    fn bad_rows_are_reported_with_their_line() {
        let text = "type,title,status,progress\nanime,Good,planned,0\nbook,Bad,planned,0\nanime,,planned,0\n";

        let parsed = parse(text.as_bytes()).expect("parse");

        assert_eq!(parsed.rows.len(), 1);
        let lines: Vec<_> = parsed.rejected.iter().map(|r| r.line).collect();
        assert_eq!(lines, [3, 4]);
    }

    #[test]
    fn headers_are_case_insensitive_and_a_bom_is_ignored() {
        let text = "\u{feff}Type, Title ,STATUS\nanime,Frieren,planned\n";

        let parsed = parse(text.as_bytes()).expect("parse");

        assert_eq!(parsed.rows.len(), 1);
    }

    #[test]
    fn files_without_the_needed_columns_are_refused() {
        assert!(parse(b"name,kind\nx,y\n").is_err());
        assert!(parse(b"").is_err());
    }

    #[test]
    fn too_many_rows_are_refused() {
        let mut text = String::from("type,title,status\n");
        for _ in 0..=MAX_ROWS {
            text.push_str("anime,A,planned\n");
        }

        assert!(parse(text.as_bytes()).is_err());
    }

    #[test]
    fn short_and_unreadable_lines_do_not_stop_the_import() {
        let mut bytes = b"type,title,status\nanime,Short\n".to_vec();
        bytes.extend_from_slice(b"anime,\xFF\xFE,planned\nanime,Fine,planned\n");

        let parsed = parse(&bytes).expect("parse");

        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.rejected.len(), 2);
    }
}
