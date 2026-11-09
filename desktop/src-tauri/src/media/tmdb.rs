use super::http::{HttpClient, HttpRequest};
use super::provider::{Metadata, MetadataProvider};
use crate::domain::media::{ItemDetails, MediaType};
use crate::error::{AppError, AppResult};
use serde::Deserialize;

const BASE_URL: &str = "https://api.themoviedb.org/3";
const IMAGE_URL: &str = "https://image.tmdb.org/t/p/w500";
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

/// Movie and show ids overlap on TMDB, so the stored id names the kind: `movie/603`, `tv/1399`.
pub struct Tmdb<'a, H: HttpClient> {
    http: &'a H,
    token: &'a str,
}

impl<'a, H: HttpClient> Tmdb<'a, H> {
    pub fn new(http: &'a H, token: &'a str) -> Self {
        Self { http, token }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        params: Vec<(&'static str, String)>,
    ) -> AppResult<T> {
        let request = HttpRequest {
            url: format!("{BASE_URL}/{path}"),
            params,
            headers: vec![("authorization", format!("Bearer {}", self.token))],
            body: None,
            max_bytes: MAX_RESPONSE_BYTES,
        };
        let body = self.http.send(request).await?.into_body()?;
        serde_json::from_slice(&body)
            .map_err(|_| AppError::Provider("TMDB sent an answer that could not be read".into()))
    }
}

impl<H: HttpClient> MetadataProvider for Tmdb<'_, H> {
    async fn search(&self, query: &str, media_type: MediaType) -> AppResult<Vec<Metadata>> {
        let kind = match media_type {
            MediaType::Movie => Kind::Movie,
            MediaType::Show => Kind::Tv,
            _ => return Err(AppError::Invalid("TMDB only has movies and shows")),
        };
        let page: SearchPage = self
            .get(
                format!("search/{}", kind.path()),
                vec![("query", query.to_string())],
            )
            .await?;
        Ok(page
            .results
            .into_iter()
            .filter_map(|title| title.into_metadata(kind))
            .collect())
    }

    async fn details(&self, external_id: &str) -> AppResult<Metadata> {
        let (kind, id) = Kind::parse(external_id)?;
        let title: Title = self
            .get(format!("{}/{id}", kind.path()), Vec::new())
            .await?;
        title.into_metadata(kind).ok_or(AppError::NotFound("title"))
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Movie,
    Tv,
}

impl Kind {
    fn path(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Tv => "tv",
        }
    }

    fn parse(external_id: &str) -> AppResult<(Self, u64)> {
        let invalid = || AppError::Invalid("the TMDB id is not valid");
        let (kind, id) = external_id.split_once('/').ok_or_else(invalid)?;
        let kind = match kind {
            "movie" => Self::Movie,
            "tv" => Self::Tv,
            _ => return Err(invalid()),
        };
        let id = id.parse().map_err(|_| invalid())?;
        Ok((kind, id))
    }
}

#[derive(Deserialize)]
struct SearchPage {
    #[serde(default)]
    results: Vec<Title>,
}

// movies and shows differ only in these field names
#[derive(Deserialize)]
struct Title {
    id: Option<u64>,
    #[serde(alias = "name")]
    title: Option<String>,
    #[serde(alias = "first_air_date")]
    release_date: Option<String>,
    poster_path: Option<String>,
    overview: Option<String>,
    genres: Option<Vec<Genre>>,
    number_of_episodes: Option<i64>,
}

#[derive(Deserialize)]
struct Genre {
    name: Option<String>,
}

impl Title {
    // a hit without an id or a title is of no use, so it is skipped
    fn into_metadata(self, kind: Kind) -> Option<Metadata> {
        let id = self.id?;
        let title = self.title.filter(|title| !title.trim().is_empty())?;
        let progress_total = match kind {
            Kind::Movie => Some(1),
            Kind::Tv => self.number_of_episodes.filter(|total| *total > 0),
        };
        Some(Metadata {
            external_id: format!("{}/{id}", kind.path()),
            details: ItemDetails {
                title,
                cover_url: self
                    .poster_path
                    .filter(|path| path.starts_with('/'))
                    .map(|path| format!("{IMAGE_URL}{path}")),
                year: self.release_date.as_deref().and_then(year_of),
                genres: self
                    .genres
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|genre| genre.name)
                    .collect(),
                synopsis: self.overview.unwrap_or_default().trim().to_string(),
                progress_total,
            },
        })
    }
}

/// The year of a `YYYY-MM-DD` date; empty strings are common for unreleased titles.
fn year_of(date: &str) -> Option<i64> {
    date.get(..4)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::test_support::FakeHttp;

    const MOVIES: &str = include_str!("fixtures/tmdb_search_movie.json");
    const SHOWS: &str = include_str!("fixtures/tmdb_search_tv.json");
    const MOVIE: &str = include_str!("fixtures/tmdb_movie_details.json");
    const SHOW: &str = include_str!("fixtures/tmdb_tv_details.json");

    #[tokio::test]
    async fn movie_search_maps_hits_and_skips_unusable_ones() {
        let http = FakeHttp::answering(200, MOVIES);

        let hits = Tmdb::new(&http, "tok")
            .search("matrix", MediaType::Movie)
            .await
            .expect("search");

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].external_id, "movie/603");
        assert_eq!(hits[0].details.title, "The Matrix");
        assert_eq!(hits[0].details.year, Some(1999));
        assert_eq!(hits[0].details.progress_total, Some(1));
        assert_eq!(
            hits[0].details.cover_url.as_deref(),
            Some("https://image.tmdb.org/t/p/w500/matrix.jpg")
        );
        assert_eq!(hits[1].details.year, None);
        assert_eq!(hits[1].details.cover_url, None);
    }

    #[tokio::test]
    async fn show_search_reads_name_and_air_date() {
        let http = FakeHttp::answering(200, SHOWS);

        let hits = Tmdb::new(&http, "tok")
            .search("thrones", MediaType::Show)
            .await
            .expect("search");

        assert_eq!(hits[0].external_id, "tv/1399");
        assert_eq!(hits[0].details.title, "Game of Thrones");
        assert_eq!(hits[0].details.year, Some(2011));
        assert_eq!(
            http.requests.borrow()[0].url,
            "https://api.themoviedb.org/3/search/tv"
        );
    }

    #[tokio::test]
    async fn the_token_goes_in_a_header_never_the_address() {
        let http = FakeHttp::answering(200, MOVIES);

        Tmdb::new(&http, "tok")
            .search("matrix", MediaType::Movie)
            .await
            .expect("search");

        let request = &http.requests.borrow()[0];
        assert_eq!(
            request.headers,
            [("authorization", "Bearer tok".to_string())]
        );
        assert_eq!(request.params, [("query", "matrix".to_string())]);
        assert!(!request.url.contains("tok"));
    }

    #[tokio::test]
    async fn show_details_have_genres_and_the_episode_total() {
        let http = FakeHttp::answering(200, SHOW);

        let show = Tmdb::new(&http, "tok")
            .details("tv/1399")
            .await
            .expect("details");

        assert_eq!(show.details.genres, ["Drama", "Sci-Fi & Fantasy"]);
        assert_eq!(show.details.progress_total, Some(73));
        assert_eq!(
            http.requests.borrow()[0].url,
            "https://api.themoviedb.org/3/tv/1399"
        );
    }

    #[tokio::test]
    async fn movie_details_use_the_movie_endpoint() {
        let http = FakeHttp::answering(200, MOVIE);

        let movie = Tmdb::new(&http, "tok")
            .details("movie/603")
            .await
            .expect("details");

        assert_eq!(movie.external_id, "movie/603");
        assert_eq!(movie.details.genres, ["Action"]);
        assert_eq!(
            http.requests.borrow()[0].url,
            "https://api.themoviedb.org/3/movie/603"
        );
    }

    #[tokio::test]
    async fn errors_are_readable_and_never_show_the_token() {
        let http = FakeHttp::answering(401, r#"{"status_code":7}"#);
        let denied = Tmdb::new(&http, "tok")
            .search("a", MediaType::Movie)
            .await
            .expect_err("denied");
        assert!(denied.to_string().contains("API key"));
        assert!(!denied.to_string().contains("tok"));

        let http = FakeHttp::answering(429, "{}");
        let limited = Tmdb::new(&http, "tok")
            .search("a", MediaType::Movie)
            .await
            .expect_err("limited");
        assert!(limited.to_string().contains("rate limited"));
    }

    #[tokio::test]
    async fn malformed_answers_and_bad_ids_are_rejected() {
        for body in ["nope", r#"{"results":"x"}"#] {
            let http = FakeHttp::answering(200, body);
            let result = Tmdb::new(&http, "t").search("a", MediaType::Movie).await;
            assert!(matches!(result, Err(AppError::Provider(_))), "{body}");
        }

        let http = FakeHttp::default();
        let provider = Tmdb::new(&http, "t");
        assert!(provider.search("a", MediaType::Anime).await.is_err());
        for id in ["603", "movie/", "film/1", "movie/../x", "tv/1/2"] {
            assert!(provider.details(id).await.is_err(), "{id}");
        }
        assert!(http.requests.borrow().is_empty());
    }

    #[test]
    fn years_come_from_the_date() {
        assert_eq!(year_of("1999-03-30"), Some(1999));
        assert_eq!(year_of(""), None);
        assert_eq!(year_of("tba"), None);
    }
}
