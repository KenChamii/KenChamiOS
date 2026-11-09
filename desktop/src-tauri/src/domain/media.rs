use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum MediaType {
    Anime,
    Manga,
    Game,
    Movie,
    Show,
    Album,
    Artist,
}

impl MediaType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anime => "anime",
            Self::Manga => "manga",
            Self::Game => "game",
            Self::Movie => "movie",
            Self::Show => "show",
            Self::Album => "album",
            Self::Artist => "artist",
        }
    }

    pub fn source(self) -> MediaSource {
        match self {
            Self::Anime | Self::Manga => MediaSource::Anilist,
            Self::Game => MediaSource::Rawg,
            Self::Movie | Self::Show => MediaSource::Tmdb,
            Self::Album | Self::Artist => MediaSource::Musicbrainz,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum MediaStatus {
    InProgress,
    Planned,
    Completed,
    OnHold,
    Dropped,
}

impl MediaStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Planned => "planned",
            Self::Completed => "completed",
            Self::OnHold => "on_hold",
            Self::Dropped => "dropped",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum MediaSource {
    Anilist,
    Rawg,
    Tmdb,
    Musicbrainz,
    Manual,
}

impl MediaSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anilist => "anilist",
            Self::Rawg => "rawg",
            Self::Tmdb => "tmdb",
            Self::Musicbrainz => "musicbrainz",
            Self::Manual => "manual",
        }
    }
}

/// A library entry together with the item it tracks.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct LibraryEntry {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub media_item_id: String,
    #[sqlx(rename = "type")]
    pub media_type: MediaType,
    pub title: String,
    pub cover_url: Option<String>,
    pub cover_cache_path: Option<String>,
    /// absolute path of the cached cover, filled in by the command layer
    #[sqlx(skip)]
    pub cover_path: Option<String>,
    pub year: Option<i64>,
    pub genres: Json<Vec<String>>,
    pub synopsis: String,
    pub source: MediaSource,
    pub external_id: Option<String>,
    pub status: MediaStatus,
    pub progress: i64,
    pub progress_total: Option<i64>,
    pub rating: Option<i64>,
    pub notes: String,
    pub tags: Json<Vec<String>>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub platform: Option<String>,
}

/// The fields of an item that are shown, whether they came from a provider or were typed in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDetails {
    pub title: String,
    pub cover_url: Option<String>,
    pub year: Option<i64>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub synopsis: String,
    pub progress_total: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManualInput {
    pub media_type: MediaType,
    pub status: MediaStatus,
    #[serde(flatten)]
    pub details: ItemDetails,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntryInput {
    pub status: MediaStatus,
    pub progress_total: Option<i64>,
    pub rating: Option<i64>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub platform: Option<String>,
}

impl EntryInput {
    pub fn validate(&self) -> AppResult<()> {
        if self.rating.is_some_and(|r| !(1..=10).contains(&r)) {
            return Err(AppError::Invalid("the rating must be 1 to 10"));
        }
        if self.progress_total.is_some_and(|t| t < 0) {
            return Err(AppError::Invalid("the total cannot be negative"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> EntryInput {
        EntryInput {
            status: MediaStatus::Planned,
            progress_total: None,
            rating: None,
            notes: String::new(),
            tags: Vec::new(),
            started_at: None,
            finished_at: None,
            platform: None,
        }
    }

    #[test]
    fn rating_must_be_one_to_ten() {
        for rating in [0, 11, -1] {
            let bad = EntryInput {
                rating: Some(rating),
                ..input()
            };
            assert!(bad.validate().is_err(), "{rating}");
        }
        let good = EntryInput {
            rating: Some(10),
            ..input()
        };
        assert!(good.validate().is_ok());
    }

    #[test]
    fn total_cannot_be_negative() {
        let bad = EntryInput {
            progress_total: Some(-1),
            ..input()
        };
        assert!(bad.validate().is_err());
    }
}
