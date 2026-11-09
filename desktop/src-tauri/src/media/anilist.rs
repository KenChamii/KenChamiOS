use super::http::{HttpClient, HttpRequest};
use super::provider::{Metadata, MetadataProvider};
use super::text::plain_text;
use crate::domain::media::{ItemDetails, MediaType};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use serde_json::json;

const ENDPOINT: &str = "https://graphql.anilist.co";
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const PER_PAGE: u32 = 20;

const MEDIA_FIELDS: &str = "id title { romaji english } coverImage { large } episodes chapters \
     genres description(asHtml: false) startDate { year }";

pub struct AniList<'a, H: HttpClient> {
    http: &'a H,
}

impl<'a, H: HttpClient> AniList<'a, H> {
    pub fn new(http: &'a H) -> Self {
        Self { http }
    }

    async fn query(&self, query: String, variables: serde_json::Value) -> AppResult<Vec<u8>> {
        let request = HttpRequest {
            url: ENDPOINT.to_string(),
            params: Vec::new(),
            body: Some(json!({ "query": query, "variables": variables }).to_string()),
            max_bytes: MAX_RESPONSE_BYTES,
        };
        self.http.send(request).await?.into_body()
    }
}

impl<H: HttpClient> MetadataProvider for AniList<'_, H> {
    async fn search(&self, query: &str, media_type: MediaType) -> AppResult<Vec<Metadata>> {
        let kind = match media_type {
            MediaType::Anime => "ANIME",
            MediaType::Manga => "MANGA",
            MediaType::Game => return Err(AppError::Invalid("AniList has no games")),
        };
        let graphql = format!(
            "query ($search: String, $type: MediaType) {{
               Page(page: 1, perPage: {PER_PAGE}) {{
                 media(search: $search, type: $type, sort: SEARCH_MATCH) {{ {MEDIA_FIELDS} }}
               }}
             }}"
        );
        let body = self
            .query(graphql, json!({ "search": query, "type": kind }))
            .await?;
        let reply: Reply<PageData> = parse(&body)?;
        let media = reply.into_data()?.page.media;
        Ok(media.into_iter().filter_map(Media::into_metadata).collect())
    }

    async fn details(&self, external_id: &str) -> AppResult<Metadata> {
        let id: u64 = external_id
            .parse()
            .map_err(|_| AppError::Invalid("the AniList id must be a number"))?;
        let graphql = format!("query ($id: Int) {{ Media(id: $id) {{ {MEDIA_FIELDS} }} }}");
        let body = self.query(graphql, json!({ "id": id })).await?;
        let reply: Reply<MediaData> = parse(&body)?;
        reply
            .into_data()?
            .media
            .and_then(Media::into_metadata)
            .ok_or(AppError::NotFound("title"))
    }
}

fn parse<T: for<'de> Deserialize<'de>>(body: &[u8]) -> AppResult<T> {
    serde_json::from_slice(body)
        .map_err(|_| AppError::Provider("AniList sent an answer that could not be read".into()))
}

#[derive(Deserialize)]
struct Reply<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<ReplyError>,
}

#[derive(Deserialize)]
struct ReplyError {
    message: Option<String>,
}

impl<T> Reply<T> {
    fn into_data(self) -> AppResult<T> {
        if let Some(error) = self.errors.into_iter().next() {
            let message = error.message.unwrap_or_else(|| "unknown error".into());
            return Err(AppError::Provider(format!("AniList: {message}")));
        }
        self.data
            .ok_or_else(|| AppError::Provider("AniList sent an empty answer".into()))
    }
}

#[derive(Deserialize)]
struct PageData {
    #[serde(rename = "Page")]
    page: Page,
}

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    media: Vec<Media>,
}

#[derive(Deserialize)]
struct MediaData {
    #[serde(rename = "Media")]
    media: Option<Media>,
}

#[derive(Deserialize)]
struct Media {
    id: Option<u64>,
    title: Option<Title>,
    #[serde(rename = "coverImage")]
    cover_image: Option<CoverImage>,
    episodes: Option<i64>,
    chapters: Option<i64>,
    genres: Option<Vec<String>>,
    description: Option<String>,
    #[serde(rename = "startDate")]
    start_date: Option<StartDate>,
}

#[derive(Deserialize)]
struct Title {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Deserialize)]
struct CoverImage {
    large: Option<String>,
}

#[derive(Deserialize)]
struct StartDate {
    year: Option<i64>,
}

impl Media {
    // a title without an id or a name is of no use, so it is skipped
    fn into_metadata(self) -> Option<Metadata> {
        let id = self.id?;
        let title = self.title?;
        let name = [title.english, title.romaji]
            .into_iter()
            .flatten()
            .find(|name| !name.trim().is_empty())?;
        Some(Metadata {
            external_id: id.to_string(),
            details: ItemDetails {
                title: name,
                cover_url: self.cover_image.and_then(|cover| cover.large),
                year: self.start_date.and_then(|date| date.year),
                genres: self.genres.unwrap_or_default(),
                synopsis: plain_text(&self.description.unwrap_or_default()),
                progress_total: self.episodes.or(self.chapters).filter(|total| *total > 0),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::test_support::FakeHttp;

    const SEARCH: &str = include_str!("fixtures/anilist_search.json");
    const DETAILS: &str = include_str!("fixtures/anilist_details.json");

    #[tokio::test]
    async fn search_maps_titles_and_skips_unusable_hits() {
        let http = FakeHttp::answering(200, SEARCH);

        let hits = AniList::new(&http)
            .search("frieren", MediaType::Anime)
            .await
            .expect("search");

        assert_eq!(hits.len(), 2);
        let first = &hits[0];
        assert_eq!(first.external_id, "154587");
        assert_eq!(first.details.title, "Frieren: Beyond Journey's End");
        assert_eq!(first.details.year, Some(2023));
        assert_eq!(first.details.progress_total, Some(28));
        assert_eq!(first.details.genres, ["Adventure", "Drama", "Fantasy"]);
        assert_eq!(
            first.details.cover_url.as_deref(),
            Some("https://s4.anilist.co/file/anilistcdn/media/anime/cover/large/154587.jpg")
        );
        assert_eq!(first.details.synopsis, "The mage Frieren\nlooks back.");
        // english title is missing, so the romaji one is used
        assert_eq!(hits[1].details.title, "Sousou no Frieren");
        assert_eq!(hits[1].details.progress_total, None);
    }

    #[tokio::test]
    async fn search_sends_the_documented_variables() {
        let http = FakeHttp::answering(200, SEARCH);

        AniList::new(&http)
            .search("one piece", MediaType::Manga)
            .await
            .expect("search");

        let requests = http.requests.borrow();
        assert_eq!(requests[0].url, ENDPOINT);
        let body: serde_json::Value =
            serde_json::from_str(requests[0].body.as_deref().expect("body")).expect("json");
        assert_eq!(body["variables"]["search"], "one piece");
        assert_eq!(body["variables"]["type"], "MANGA");
        assert!(
            body["query"]
                .as_str()
                .is_some_and(|q| q.contains("Page(page: 1"))
        );
    }

    #[tokio::test]
    async fn details_maps_one_title() {
        let http = FakeHttp::answering(200, DETAILS);

        let found = AniList::new(&http).details("30013").await.expect("details");

        assert_eq!(found.external_id, "30013");
        assert_eq!(found.details.title, "One Piece");
        assert_eq!(found.details.progress_total, None);
    }

    #[tokio::test]
    async fn a_missing_title_is_not_found() {
        let http = FakeHttp::answering(200, r#"{"data":{"Media":null}}"#);
        let result = AniList::new(&http).details("1").await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn empty_results_are_an_empty_list() {
        let http = FakeHttp::answering(200, r#"{"data":{"Page":{"media":[]}}}"#);
        let hits = AniList::new(&http)
            .search("zzzz", MediaType::Anime)
            .await
            .expect("search");
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn rate_limits_and_graphql_errors_are_readable() {
        let http = FakeHttp::answering(429, r#"{"errors":[{"message":"Too Many Requests."}]}"#);
        let limited = AniList::new(&http).search("a", MediaType::Anime).await;
        assert!(
            limited
                .expect_err("limited")
                .to_string()
                .contains("rate limited")
        );

        let http = FakeHttp::answering(200, r#"{"data":null,"errors":[{"message":"Not Found."}]}"#);
        let failed = AniList::new(&http).search("a", MediaType::Anime).await;
        assert!(
            failed
                .expect_err("failed")
                .to_string()
                .contains("Not Found.")
        );
    }

    #[tokio::test]
    async fn malformed_answers_are_an_error() {
        for body in [
            "not json",
            r#"{"unexpected":true}"#,
            r#"{"data":{"Page":5}}"#,
        ] {
            let http = FakeHttp::answering(200, body);
            let result = AniList::new(&http).search("a", MediaType::Anime).await;
            assert!(matches!(result, Err(AppError::Provider(_))), "{body}");
        }
    }

    #[tokio::test]
    async fn games_and_bad_ids_are_rejected_without_a_request() {
        let http = FakeHttp::default();
        let provider = AniList::new(&http);

        assert!(provider.search("a", MediaType::Game).await.is_err());
        assert!(provider.details("1; drop").await.is_err());
        assert!(http.requests.borrow().is_empty());
    }
}
