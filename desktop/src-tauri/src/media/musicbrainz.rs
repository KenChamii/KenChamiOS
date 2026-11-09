use super::http::{HttpClient, HttpRequest};
use super::pacing::{Clock, Limiter};
use super::provider::{Metadata, MetadataProvider};
use crate::domain::media::{ItemDetails, MediaType};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use uuid::Uuid;

const BASE_URL: &str = "https://musicbrainz.org/ws/2";
const COVER_URL: &str = "https://coverartarchive.org/release-group";
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const PAGE_SIZE: &str = "20";
const MAX_TAGS: usize = 5;

/// Albums are release groups and artists are artists; the stored id names the kind:
/// `release-group/<mbid>` or `artist/<mbid>`. Every request waits its turn in the limiter.
pub struct MusicBrainz<'a, H: HttpClient, C: Clock> {
    http: &'a H,
    limiter: &'a Limiter<C>,
}

impl<'a, H: HttpClient, C: Clock> MusicBrainz<'a, H, C> {
    pub fn new(http: &'a H, limiter: &'a Limiter<C>) -> Self {
        Self { http, limiter }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        mut params: Vec<(&'static str, String)>,
    ) -> AppResult<T> {
        params.push(("fmt", "json".to_string()));
        let request = HttpRequest {
            url: format!("{BASE_URL}/{path}"),
            params,
            headers: Vec::new(),
            body: None,
            max_bytes: MAX_RESPONSE_BYTES,
        };
        self.limiter.wait().await;
        let body = self.http.send(request).await?.into_body()?;
        serde_json::from_slice(&body).map_err(|_| {
            AppError::Provider("MusicBrainz sent an answer that could not be read".into())
        })
    }
}

impl<H: HttpClient, C: Clock> MetadataProvider for MusicBrainz<'_, H, C> {
    async fn search(&self, query: &str, media_type: MediaType) -> AppResult<Vec<Metadata>> {
        let kind = match media_type {
            MediaType::Album => Kind::Album,
            MediaType::Artist => Kind::Artist,
            _ => return Err(AppError::Invalid("MusicBrainz only has albums and artists")),
        };
        let params = vec![
            ("query", query.to_string()),
            ("limit", PAGE_SIZE.to_string()),
        ];
        let page: SearchPage = self.get(kind.path().to_string(), params).await?;
        let hits = match kind {
            Kind::Album => page.release_groups,
            Kind::Artist => page.artists,
        };
        Ok(hits
            .into_iter()
            .filter_map(|hit| hit.into_metadata(kind))
            .collect())
    }

    async fn details(&self, external_id: &str) -> AppResult<Metadata> {
        let (kind, id) = Kind::parse(external_id)?;
        let inc = match kind {
            Kind::Album => "tags+artist-credits",
            Kind::Artist => "tags",
        };
        let entity: Entity = self
            .get(
                format!("{}/{id}", kind.path()),
                vec![("inc", inc.to_string())],
            )
            .await?;
        entity
            .into_metadata(kind)
            .ok_or(AppError::NotFound("music title"))
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Album,
    Artist,
}

impl Kind {
    fn path(self) -> &'static str {
        match self {
            Self::Album => "release-group",
            Self::Artist => "artist",
        }
    }

    fn parse(external_id: &str) -> AppResult<(Self, Uuid)> {
        let invalid = || AppError::Invalid("the MusicBrainz id is not valid");
        let (kind, id) = external_id.split_once('/').ok_or_else(invalid)?;
        let kind = match kind {
            "release-group" => Self::Album,
            "artist" => Self::Artist,
            _ => return Err(invalid()),
        };
        Ok((kind, Uuid::parse_str(id).map_err(|_| invalid())?))
    }
}

// the search answer names the list after the entity, so both are read and one stays empty
#[derive(Deserialize)]
struct SearchPage {
    #[serde(default, rename = "release-groups")]
    release_groups: Vec<Entity>,
    #[serde(default)]
    artists: Vec<Entity>,
}

// albums and artists share one shape; fields that do not apply are simply absent
#[derive(Deserialize)]
struct Entity {
    id: Option<String>,
    title: Option<String>,
    name: Option<String>,
    #[serde(rename = "first-release-date")]
    first_release_date: Option<String>,
    #[serde(rename = "primary-type")]
    primary_type: Option<String>,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<Credit>>,
    tags: Option<Vec<Tag>>,
    country: Option<String>,
    disambiguation: Option<String>,
    #[serde(rename = "life-span")]
    life_span: Option<LifeSpan>,
}

#[derive(Deserialize)]
struct Credit {
    name: Option<String>,
    artist: Option<CreditArtist>,
}

#[derive(Deserialize)]
struct CreditArtist {
    name: Option<String>,
}

#[derive(Deserialize)]
struct Tag {
    name: Option<String>,
}

#[derive(Deserialize)]
struct LifeSpan {
    begin: Option<String>,
}

impl Entity {
    // a hit without a valid id or a name is of no use, so it is skipped
    fn into_metadata(self, kind: Kind) -> Option<Metadata> {
        let id = Uuid::parse_str(self.id.as_deref()?).ok()?;
        let name = match kind {
            Kind::Album => self.title,
            Kind::Artist => self.name,
        }
        .filter(|name| !name.trim().is_empty())?;
        let genres = self
            .tags
            .unwrap_or_default()
            .into_iter()
            .filter_map(|tag| tag.name)
            .take(MAX_TAGS)
            .collect();
        let (cover_url, date, synopsis) = match kind {
            Kind::Album => {
                let artists = credited_artists(self.artist_credit.unwrap_or_default());
                let synopsis = match (self.primary_type, artists.is_empty()) {
                    (Some(kind), false) => format!("{kind} by {artists}"),
                    (Some(kind), true) => kind,
                    (None, false) => format!("By {artists}"),
                    (None, true) => String::new(),
                };
                (
                    Some(format!("{COVER_URL}/{id}/front-250")),
                    self.first_release_date,
                    synopsis,
                )
            }
            Kind::Artist => {
                let synopsis = [self.disambiguation, self.country]
                    .into_iter()
                    .flatten()
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                (None, self.life_span.and_then(|span| span.begin), synopsis)
            }
        };
        Some(Metadata {
            external_id: format!("{}/{id}", kind.path()),
            details: ItemDetails {
                title: name,
                cover_url,
                year: date.as_deref().and_then(year_of),
                genres,
                synopsis,
                progress_total: None,
            },
        })
    }
}

fn credited_artists(credits: Vec<Credit>) -> String {
    credits
        .into_iter()
        .filter_map(|credit| credit.name.or_else(|| credit.artist?.name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The year of a `YYYY`, `YYYY-MM` or `YYYY-MM-DD` date; empty strings are common.
fn year_of(date: &str) -> Option<i64> {
    date.get(..4)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::pacing::tests::FakeClock;
    use crate::media::test_support::FakeHttp;
    use std::time::Duration;

    const ALBUMS: &str = include_str!("fixtures/musicbrainz_release_groups.json");
    const ARTISTS: &str = include_str!("fixtures/musicbrainz_artists.json");
    const ALBUM_ID: &str = "release-group/b1392450-e666-3926-a536-22c65f834433";
    const ARTIST_ID: &str = "artist/a74b1b7f-71a5-4011-9441-d0b5e4122711";

    fn limiter() -> Limiter<FakeClock> {
        Limiter::new(FakeClock::default(), Duration::from_millis(1100))
    }

    #[tokio::test]
    async fn album_search_maps_hits_and_builds_the_cover_address() {
        let http = FakeHttp::answering(200, ALBUMS);
        let limiter = limiter();

        let hits = MusicBrainz::new(&http, &limiter)
            .search("ok computer", MediaType::Album)
            .await
            .expect("search");

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].external_id, ALBUM_ID);
        assert_eq!(hits[0].details.title, "OK Computer");
        assert_eq!(hits[0].details.year, Some(1997));
        assert_eq!(hits[0].details.synopsis, "Album by Radiohead");
        assert_eq!(hits[0].details.genres, ["alternative rock", "art rock"]);
        assert_eq!(
            hits[0].details.cover_url.as_deref(),
            Some(
                "https://coverartarchive.org/release-group/b1392450-e666-3926-a536-22c65f834433/front-250"
            )
        );
        assert_eq!(hits[1].details.year, None);
        assert!(hits[1].details.genres.is_empty());
    }

    #[tokio::test]
    async fn artist_search_maps_hits() {
        let http = FakeHttp::answering(200, ARTISTS);
        let limiter = limiter();

        let hits = MusicBrainz::new(&http, &limiter)
            .search("radiohead", MediaType::Artist)
            .await
            .expect("search");

        assert_eq!(hits[0].external_id, ARTIST_ID);
        assert_eq!(hits[0].details.title, "Radiohead");
        assert_eq!(hits[0].details.year, Some(1985));
        assert_eq!(hits[0].details.synopsis, "English rock band, GB");
        assert_eq!(hits[0].details.cover_url, None);
        let request = &http.requests.borrow()[0];
        assert_eq!(request.url, "https://musicbrainz.org/ws/2/artist");
        assert!(request.params.contains(&("fmt", "json".to_string())));
        assert!(request.params.contains(&("query", "radiohead".to_string())));
    }

    #[tokio::test]
    async fn details_look_up_by_kind_and_id() {
        let http = FakeHttp::answering(
            200,
            r#"{"id":"b1392450-e666-3926-a536-22c65f834433","title":"OK Computer"}"#,
        );
        let limiter = limiter();

        let album = MusicBrainz::new(&http, &limiter)
            .details(ALBUM_ID)
            .await
            .expect("details");

        assert_eq!(album.details.title, "OK Computer");
        let request = &http.requests.borrow()[0];
        assert_eq!(
            request.url,
            "https://musicbrainz.org/ws/2/release-group/b1392450-e666-3926-a536-22c65f834433"
        );
        assert!(
            request
                .params
                .contains(&("inc", "tags+artist-credits".to_string()))
        );
    }

    #[tokio::test]
    async fn requests_are_spaced_by_the_limiter_without_real_waiting() {
        let http = FakeHttp::answering(200, ARTISTS);
        http.push(200, ARTISTS);
        http.push(200, ARTISTS);
        let limiter = limiter();
        let provider = MusicBrainz::new(&http, &limiter);

        for _ in 0..3 {
            provider
                .search("radiohead", MediaType::Artist)
                .await
                .expect("search");
        }

        assert_eq!(limiter_sleeps(&limiter), 2);
    }

    fn limiter_sleeps(limiter: &Limiter<FakeClock>) -> usize {
        limiter.clock().sleeps.borrow().len()
    }

    #[tokio::test]
    async fn errors_are_readable() {
        let http = FakeHttp::answering(503, "");
        let limiter = limiter();
        let error = MusicBrainz::new(&http, &limiter)
            .search("a", MediaType::Artist)
            .await
            .expect_err("busy");
        assert!(error.to_string().contains("503"));

        let http = FakeHttp::answering(200, "nope");
        let result = MusicBrainz::new(&http, &limiter)
            .search("a", MediaType::Artist)
            .await;
        assert!(matches!(result, Err(AppError::Provider(_))));
    }

    #[tokio::test]
    async fn wrong_types_and_bad_ids_make_no_request() {
        let http = FakeHttp::default();
        let limiter = limiter();
        let provider = MusicBrainz::new(&http, &limiter);

        assert!(provider.search("a", MediaType::Game).await.is_err());
        for id in [
            "b1392450-e666-3926-a536-22c65f834433",
            "release-group/../x",
            "track/b1392450-e666-3926-a536-22c65f834433",
            "artist/1",
        ] {
            assert!(provider.details(id).await.is_err(), "{id}");
        }
        assert!(http.requests.borrow().is_empty());
    }
}
