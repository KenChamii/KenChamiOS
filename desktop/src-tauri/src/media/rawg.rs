use super::http::{HttpClient, HttpRequest};
use super::provider::{Metadata, MetadataProvider};
use super::text::plain_text;
use crate::domain::media::{ItemDetails, MediaType};
use crate::error::{AppError, AppResult};
use serde::Deserialize;

const BASE_URL: &str = "https://api.rawg.io/api/games";
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const PAGE_SIZE: &str = "20";

pub struct Rawg<'a, H: HttpClient> {
    http: &'a H,
    key: &'a str,
}

impl<'a, H: HttpClient> Rawg<'a, H> {
    pub fn new(http: &'a H, key: &'a str) -> Self {
        Self { http, key }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        url: String,
        mut params: Vec<(&'static str, String)>,
    ) -> AppResult<T> {
        params.push(("key", self.key.to_string()));
        let request = HttpRequest {
            url,
            params,
            headers: Vec::new(),
            body: None,
            max_bytes: MAX_RESPONSE_BYTES,
        };
        let body = self.http.send(request).await?.into_body()?;
        serde_json::from_slice(&body)
            .map_err(|_| AppError::Provider("RAWG sent an answer that could not be read".into()))
    }
}

impl<H: HttpClient> MetadataProvider for Rawg<'_, H> {
    async fn search(&self, query: &str, media_type: MediaType) -> AppResult<Vec<Metadata>> {
        if media_type != MediaType::Game {
            return Err(AppError::Invalid("RAWG only has games"));
        }
        let params = vec![
            ("search", query.to_string()),
            ("page_size", PAGE_SIZE.to_string()),
        ];
        let page: SearchPage = self.get(BASE_URL.to_string(), params).await?;
        Ok(page
            .results
            .into_iter()
            .filter_map(Game::into_metadata)
            .collect())
    }

    async fn details(&self, external_id: &str) -> AppResult<Metadata> {
        let id: u64 = external_id
            .parse()
            .map_err(|_| AppError::Invalid("the RAWG id must be a number"))?;
        let game: Game = self.get(format!("{BASE_URL}/{id}"), Vec::new()).await?;
        game.into_metadata().ok_or(AppError::NotFound("game"))
    }
}

#[derive(Deserialize)]
struct SearchPage {
    #[serde(default)]
    results: Vec<Game>,
}

#[derive(Deserialize)]
struct Game {
    id: Option<u64>,
    name: Option<String>,
    background_image: Option<String>,
    released: Option<String>,
    genres: Option<Vec<Genre>>,
    description_raw: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct Genre {
    name: Option<String>,
}

impl Game {
    // a game without an id or a name is of no use, so it is skipped
    fn into_metadata(self) -> Option<Metadata> {
        let id = self.id?;
        let title = self.name.filter(|name| !name.trim().is_empty())?;
        let synopsis = match (self.description_raw, self.description) {
            (Some(raw), _) => raw.trim().to_string(),
            (None, Some(html)) => plain_text(&html),
            (None, None) => String::new(),
        };
        Some(Metadata {
            external_id: id.to_string(),
            details: ItemDetails {
                title,
                cover_url: self.background_image,
                year: self.released.as_deref().and_then(year_of),
                genres: self
                    .genres
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|genre| genre.name)
                    .collect(),
                synopsis,
                progress_total: None,
            },
        })
    }
}

/// The year of a `YYYY-MM-DD` release date.
fn year_of(released: &str) -> Option<i64> {
    released.get(..4)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::test_support::FakeHttp;

    const SEARCH: &str = include_str!("fixtures/rawg_search.json");
    const DETAILS: &str = include_str!("fixtures/rawg_details.json");

    #[tokio::test]
    async fn search_maps_games_and_skips_unusable_hits() {
        let http = FakeHttp::answering(200, SEARCH);

        let hits = Rawg::new(&http, "secret")
            .search("hollow knight", MediaType::Game)
            .await
            .expect("search");

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].external_id, "3328");
        assert_eq!(hits[0].details.title, "Hollow Knight");
        assert_eq!(hits[0].details.year, Some(2017));
        assert_eq!(hits[0].details.genres, ["Action", "Indie"]);
        assert_eq!(
            hits[0].details.cover_url.as_deref(),
            Some("https://media.rawg.io/media/games/hk.jpg")
        );
        assert_eq!(hits[1].details.year, None);
        assert!(hits[1].details.genres.is_empty());
    }

    #[tokio::test]
    async fn search_sends_the_key_and_query_as_parameters() {
        let http = FakeHttp::answering(200, SEARCH);

        Rawg::new(&http, "secret")
            .search("hollow knight", MediaType::Game)
            .await
            .expect("search");

        let requests = http.requests.borrow();
        assert_eq!(requests[0].url, BASE_URL);
        assert!(requests[0].body.is_none());
        let params = &requests[0].params;
        assert!(params.contains(&("search", "hollow knight".to_string())));
        assert!(params.contains(&("key", "secret".to_string())));
    }

    #[tokio::test]
    async fn details_prefers_the_plain_description() {
        let http = FakeHttp::answering(200, DETAILS);

        let game = Rawg::new(&http, "secret")
            .details("3328")
            .await
            .expect("details");

        assert_eq!(game.details.synopsis, "A knight explores a ruined kingdom.");
        assert_eq!(
            http.requests.borrow()[0].url,
            "https://api.rawg.io/api/games/3328"
        );
    }

    #[tokio::test]
    async fn html_descriptions_are_made_plain() {
        let body = r#"{"id":1,"name":"G","description":"<p>One &amp; two</p>"}"#;
        let http = FakeHttp::answering(200, body);

        let game = Rawg::new(&http, "k").details("1").await.expect("details");

        assert_eq!(game.details.synopsis, "One & two");
    }

    #[tokio::test]
    async fn errors_are_readable_and_never_show_the_key() {
        let http = FakeHttp::answering(401, r#"{"detail":"Invalid token."}"#);
        let denied = Rawg::new(&http, "secret")
            .search("a", MediaType::Game)
            .await
            .expect_err("denied");
        assert!(denied.to_string().contains("API key"));
        assert!(!denied.to_string().contains("secret"));

        let http = FakeHttp::answering(429, "{}");
        let limited = Rawg::new(&http, "secret")
            .search("a", MediaType::Game)
            .await
            .expect_err("limited");
        assert!(limited.to_string().contains("rate limited"));
    }

    #[tokio::test]
    async fn empty_and_malformed_answers_are_handled() {
        let http = FakeHttp::answering(200, r#"{"count":0,"results":[]}"#);
        let hits = Rawg::new(&http, "k")
            .search("zzzz", MediaType::Game)
            .await
            .expect("search");
        assert!(hits.is_empty());

        for body in ["nope", r#"{"results":"x"}"#] {
            let http = FakeHttp::answering(200, body);
            let result = Rawg::new(&http, "k").search("a", MediaType::Game).await;
            assert!(matches!(result, Err(AppError::Provider(_))), "{body}");
        }
    }

    #[tokio::test]
    async fn anime_and_bad_ids_are_rejected_without_a_request() {
        let http = FakeHttp::default();
        let provider = Rawg::new(&http, "k");

        assert!(provider.search("a", MediaType::Anime).await.is_err());
        assert!(provider.details("../x").await.is_err());
        assert!(http.requests.borrow().is_empty());
    }

    #[test]
    fn years_come_from_the_release_date() {
        assert_eq!(year_of("2017-02-24"), Some(2017));
        assert_eq!(year_of("tba"), None);
        assert_eq!(year_of(""), None);
    }
}
