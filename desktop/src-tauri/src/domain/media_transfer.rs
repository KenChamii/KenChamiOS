use super::civil_date;
use super::media::{ItemDetails, MediaSource, MediaStatus, MediaType};
use super::tags;
use serde::de::value::{Error as ValueError, StringDeserializer};
use serde::{Deserialize, Serialize};

pub const MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;
pub const MAX_ROWS: usize = 5000;
const MAX_TITLE: usize = 300;
const MAX_NOTES: usize = 20_000;
const MAX_SHORT: usize = 200;
const MAX_URL: usize = 2000;
const MAX_LIST: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    Csv,
    /// MyAnimeList export XML, also written by tools that convert AniList lists
    Mal,
}

/// One line of an import file as text, before it is checked.
#[derive(Debug, Default, Clone)]
pub struct RawRow {
    pub line: usize,
    pub media_type: String,
    pub title: String,
    pub year: String,
    pub status: String,
    pub progress: String,
    pub total: String,
    pub rating: String,
    pub genres: String,
    pub tags: String,
    pub notes: String,
    pub started: String,
    pub finished: String,
    pub platform: String,
    pub source: String,
    pub external_id: String,
    pub cover_url: String,
}

/// A checked row, ready to be added to the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportRow {
    pub line: usize,
    pub media_type: MediaType,
    pub status: MediaStatus,
    #[serde(flatten)]
    pub details: ItemDetails,
    pub progress: i64,
    pub rating: Option<i64>,
    pub notes: String,
    pub tags: Vec<String>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub platform: Option<String>,
    pub source: MediaSource,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RejectedRow {
    pub line: usize,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct ParsedFile {
    pub rows: Vec<ImportRow>,
    pub rejected: Vec<RejectedRow>,
}

impl ParsedFile {
    pub fn push(&mut self, raw: RawRow) {
        let line = raw.line;
        match ImportRow::check(raw) {
            Ok(row) => self.rows.push(row),
            Err(reason) => self.rejected.push(RejectedRow {
                line,
                reason: reason.to_string(),
            }),
        }
    }

    pub fn reject(&mut self, line: usize, reason: &str) {
        self.rejected.push(RejectedRow {
            line,
            reason: reason.to_string(),
        });
    }
}

#[derive(Debug, Serialize)]
pub struct PreviewRow {
    #[serde(flatten)]
    pub row: ImportRow,
    pub duplicate: bool,
}

#[derive(Debug, Serialize)]
pub struct ImportPreview {
    pub rows: Vec<PreviewRow>,
    pub rejected: Vec<RejectedRow>,
    pub new_count: usize,
    pub duplicate_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub added: usize,
    pub duplicates: usize,
    pub rejected: usize,
}

impl ImportRow {
    /// Checks every field of a raw row; the error says what is wrong in plain words.
    pub fn check(raw: RawRow) -> Result<Self, &'static str> {
        let media_type: MediaType = parse_name(&raw.media_type).ok_or("unknown type")?;
        let status: MediaStatus = parse_name(&raw.status).ok_or("unknown status")?;
        let title = raw.title.trim();
        if title.is_empty() {
            return Err("the title is empty");
        }
        if title.chars().count() > MAX_TITLE {
            return Err("the title is too long");
        }
        let total = optional_count(&raw.total).ok_or("the total is not a whole number")?;
        let progress = optional_count(&raw.progress).ok_or("the progress is not a whole number")?;
        let progress = progress.unwrap_or(0);
        if total.is_some_and(|total| progress > total) {
            return Err("the progress is past the total");
        }
        let year = optional_count(&raw.year).ok_or("the year is not a number")?;
        if year.is_some_and(|year| !(1..=9999).contains(&year)) {
            return Err("the year is out of range");
        }
        let rating = optional_count(&raw.rating).ok_or("the rating is not a number")?;
        // some exports write 0 for "not rated"
        let rating = rating.filter(|rating| *rating != 0);
        if rating.is_some_and(|rating| !(1..=10).contains(&rating)) {
            return Err("the rating must be 1 to 10");
        }
        let started_at = optional_date(&raw.started).ok_or("the start date is not valid")?;
        let finished_at = optional_date(&raw.finished).ok_or("the finish date is not valid")?;
        let (source, external_id) = origin_of(media_type, &raw)?;
        let cover_url = raw.cover_url.trim();
        if cover_url.len() > MAX_URL || !(cover_url.is_empty() || cover_url.starts_with("https://"))
        {
            return Err("the cover address must be an https address");
        }
        let genres = list_of(&raw.genres)?;
        let tags = list_of(&raw.tags)?;
        if raw.notes.chars().count() > MAX_NOTES || raw.platform.chars().count() > MAX_SHORT {
            return Err("a text field is too long");
        }
        Ok(Self {
            line: raw.line,
            media_type,
            status,
            details: ItemDetails {
                title: title.to_string(),
                cover_url: (!cover_url.is_empty()).then(|| cover_url.to_string()),
                year,
                genres,
                synopsis: String::new(),
                progress_total: total,
            },
            progress,
            rating,
            notes: raw.notes.trim().to_string(),
            tags,
            started_at,
            finished_at,
            platform: Some(raw.platform.trim().to_string()).filter(|p| !p.is_empty()),
            source,
            external_id,
        })
    }
}

fn parse_name<T: for<'de> Deserialize<'de>>(text: &str) -> Option<T> {
    T::deserialize(StringDeserializer::<ValueError>::new(
        text.trim().to_lowercase(),
    ))
    .ok()
}

/// `None` for text that is not a whole number >= 0; empty text is `Some(None)`.
fn optional_count(text: &str) -> Option<Option<i64>> {
    let text = text.trim();
    if text.is_empty() {
        return Some(None);
    }
    text.parse::<i64>().ok().filter(|n| *n >= 0).map(Some)
}

/// Empty and all-zero dates mean "not set"; anything else must be a real day.
fn optional_date(text: &str) -> Option<Option<i64>> {
    let text = text.trim();
    if text.is_empty() || text == "0000-00-00" {
        return Some(None);
    }
    civil_date::parse(text).map(Some)
}

fn origin_of(
    media_type: MediaType,
    raw: &RawRow,
) -> Result<(MediaSource, Option<String>), &'static str> {
    let external_id = raw.external_id.trim();
    if raw.source.trim().is_empty() || external_id.is_empty() {
        return Ok((MediaSource::Manual, None));
    }
    let source: MediaSource = parse_name(&raw.source).ok_or("unknown source")?;
    if source == MediaSource::Manual {
        return Ok((MediaSource::Manual, None));
    }
    if source != media_type.source() {
        return Err("the source does not match the type");
    }
    if external_id.chars().count() > MAX_SHORT {
        return Err("the external id is too long");
    }
    Ok((source, Some(external_id.to_string())))
}

fn list_of(text: &str) -> Result<Vec<String>, &'static str> {
    let items = tags::clean(text.split(';').map(str::to_string).collect());
    if items.len() > MAX_LIST || items.iter().any(|item| item.chars().count() > MAX_SHORT) {
        return Err("a list holds too many or too long items");
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw() -> RawRow {
        RawRow {
            line: 2,
            media_type: "anime".into(),
            title: "Frieren".into(),
            status: "in_progress".into(),
            ..RawRow::default()
        }
    }

    fn reason(raw: RawRow) -> &'static str {
        ImportRow::check(raw).expect_err("should be rejected")
    }

    #[test]
    fn a_minimal_row_is_accepted_with_defaults() {
        let row = ImportRow::check(raw()).expect("row");

        assert_eq!(row.media_type, MediaType::Anime);
        assert_eq!(row.status, MediaStatus::InProgress);
        assert_eq!(row.progress, 0);
        assert_eq!(row.source, MediaSource::Manual);
        assert_eq!(row.details.progress_total, None);
    }

    #[test]
    fn a_full_row_is_parsed() {
        let row = ImportRow::check(RawRow {
            media_type: " Game ".into(),
            year: "2017".into(),
            progress: "12".into(),
            total: "20".into(),
            rating: "9".into(),
            genres: "Action; Indie;Action".into(),
            tags: "cozy".into(),
            notes: " fun ".into(),
            started: "2024-01-05".into(),
            finished: "0000-00-00".into(),
            platform: "PC".into(),
            source: "rawg".into(),
            external_id: "3328".into(),
            cover_url: "https://media.rawg.io/x.jpg".into(),
            ..raw()
        })
        .expect("row");

        assert_eq!(row.media_type, MediaType::Game);
        assert_eq!(row.details.year, Some(2017));
        assert_eq!(row.details.genres, ["Action", "Indie"]);
        assert_eq!(row.progress, 12);
        assert_eq!(row.rating, Some(9));
        assert_eq!(row.started_at, civil_date::parse("2024-01-05"));
        assert_eq!(row.finished_at, None);
        assert_eq!(row.platform.as_deref(), Some("PC"));
        assert_eq!(row.external_id.as_deref(), Some("3328"));
        assert_eq!(row.notes, "fun");
    }

    #[test]
    fn a_rating_of_zero_means_not_rated() {
        let row = ImportRow::check(RawRow {
            rating: "0".into(),
            ..raw()
        })
        .expect("row");
        assert_eq!(row.rating, None);
    }

    #[test]
    fn bad_rows_say_why() {
        let bad = |change: fn(&mut RawRow)| {
            let mut row = raw();
            change(&mut row);
            reason(row)
        };
        assert_eq!(bad(|r| r.media_type = "book".into()), "unknown type");
        assert_eq!(bad(|r| r.status = "paused".into()), "unknown status");
        assert_eq!(bad(|r| r.title = "  ".into()), "the title is empty");
        assert_eq!(bad(|r| r.title = "x".repeat(301)), "the title is too long");
        assert_eq!(
            bad(|r| r.progress = "-1".into()),
            "the progress is not a whole number"
        );
        assert_eq!(
            bad(|r| {
                r.progress = "5".into();
                r.total = "4".into();
            }),
            "the progress is past the total"
        );
        assert_eq!(
            bad(|r| r.rating = "11".into()),
            "the rating must be 1 to 10"
        );
        assert_eq!(bad(|r| r.year = "0".into()), "the year is out of range");
        assert_eq!(
            bad(|r| r.started = "2024-02-30".into()),
            "the start date is not valid"
        );
        assert_eq!(
            bad(|r| r.cover_url = "http://x/y.jpg".into()),
            "the cover address must be an https address"
        );
        assert_eq!(
            bad(|r| {
                r.source = "rawg".into();
                r.external_id = "1".into();
            }),
            "the source does not match the type"
        );
        assert_eq!(
            bad(|r| r.genres = (0..31)
                .map(|n| format!("g{n}"))
                .collect::<Vec<_>>()
                .join(";")),
            "a list holds too many or too long items"
        );
    }

    #[test]
    fn a_source_without_an_id_falls_back_to_manual() {
        let row = ImportRow::check(RawRow {
            source: "anilist".into(),
            ..raw()
        })
        .expect("row");
        assert_eq!(row.source, MediaSource::Manual);
        assert_eq!(row.external_id, None);
    }

    #[test]
    fn parsed_files_collect_rows_and_rejections() {
        let mut parsed = ParsedFile::default();
        parsed.push(raw());
        parsed.push(RawRow {
            line: 3,
            title: String::new(),
            ..raw()
        });

        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(
            parsed.rejected,
            [RejectedRow {
                line: 3,
                reason: "the title is empty".into()
            }]
        );
    }
}
