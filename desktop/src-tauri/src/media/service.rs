use super::anilist::AniList;
use super::covers;
use super::http::HttpClient;
use super::musicbrainz::MusicBrainz;
use super::pacing::{Clock, Limiter};
use super::provider::{Metadata, MetadataProvider};
use super::rawg::Rawg;
use super::secrets::SecretStore;
use super::tmdb::Tmdb;
use crate::db::metadata_cache::{self, Cached};
use crate::db::{Db, now_ms};
use crate::domain::media::MediaType;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::path::Path;
use std::time::Duration;

const GAMES_KEY: &str = "rawg_api_key";
const TMDB_TOKEN: &str = "tmdb_read_token";
const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const SEARCH_TTL_MS: i64 = DAY_MS;
const DETAILS_TTL_MS: i64 = 7 * DAY_MS;
// musicbrainz allows about one request per second per ip, with a little room
const MUSIC_GAP: Duration = Duration::from_millis(1100);

/// Searches the metadata sources, answering from the local cache when it can.
pub struct MetadataService<H: HttpClient, S: SecretStore, C: Clock> {
    http: H,
    secrets: S,
    music_limiter: Limiter<C>,
}

impl<H: HttpClient, S: SecretStore, C: Clock> MetadataService<H, S, C> {
    pub fn new(http: H, secrets: S, clock: C) -> Self {
        Self {
            http,
            secrets,
            music_limiter: Limiter::new(clock, MUSIC_GAP),
        }
    }

    pub async fn search(
        &self,
        db: &Db,
        query: &str,
        media_type: MediaType,
    ) -> AppResult<Vec<Metadata>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let key = format!(
            "search:{}:{}:{}",
            media_type.source().as_str(),
            media_type.as_str(),
            query.to_lowercase()
        );
        cached(
            db,
            &key,
            SEARCH_TTL_MS,
            self.fetch_search(query, media_type),
        )
        .await
    }

    pub async fn details(
        &self,
        db: &Db,
        media_type: MediaType,
        external_id: &str,
    ) -> AppResult<Metadata> {
        let key = format!("details:{}:{external_id}", media_type.source().as_str());
        cached(
            db,
            &key,
            DETAILS_TTL_MS,
            self.fetch_details(media_type, external_id),
        )
        .await
    }

    pub async fn download_cover(
        &self,
        covers_dir: &Path,
        item_id: &str,
        url: &str,
    ) -> AppResult<String> {
        covers::download(&self.http, covers_dir, item_id, url).await
    }

    pub fn has_games_key(&self) -> AppResult<bool> {
        Ok(self.secrets.get(GAMES_KEY)?.is_some())
    }

    pub fn set_games_key(&self, key: &str) -> AppResult<()> {
        self.set_secret(GAMES_KEY, key)
    }

    pub fn clear_games_key(&self) -> AppResult<()> {
        self.secrets.clear(GAMES_KEY)
    }

    pub fn has_tmdb_token(&self) -> AppResult<bool> {
        Ok(self.secrets.get(TMDB_TOKEN)?.is_some())
    }

    pub fn set_tmdb_token(&self, token: &str) -> AppResult<()> {
        self.set_secret(TMDB_TOKEN, token)
    }

    pub fn clear_tmdb_token(&self) -> AppResult<()> {
        self.secrets.clear(TMDB_TOKEN)
    }

    fn set_secret(&self, name: &str, value: &str) -> AppResult<()> {
        let value = value.trim();
        if value.is_empty() || value.contains(char::is_whitespace) {
            return Err(AppError::Invalid(
                "the API key must not be empty or hold spaces",
            ));
        }
        self.secrets.set(name, value)
    }

    async fn fetch_search(&self, query: &str, media_type: MediaType) -> AppResult<Vec<Metadata>> {
        match media_type {
            MediaType::Game => {
                let key = self.games_key()?;
                Rawg::new(&self.http, &key).search(query, media_type).await
            }
            MediaType::Movie | MediaType::Show => {
                let token = self.tmdb_token()?;
                Tmdb::new(&self.http, &token)
                    .search(query, media_type)
                    .await
            }
            MediaType::Album | MediaType::Artist => {
                MusicBrainz::new(&self.http, &self.music_limiter)
                    .search(query, media_type)
                    .await
            }
            MediaType::Anime | MediaType::Manga => {
                AniList::new(&self.http).search(query, media_type).await
            }
        }
    }

    async fn fetch_details(&self, media_type: MediaType, external_id: &str) -> AppResult<Metadata> {
        match media_type {
            MediaType::Game => {
                let key = self.games_key()?;
                Rawg::new(&self.http, &key).details(external_id).await
            }
            MediaType::Movie | MediaType::Show => {
                let token = self.tmdb_token()?;
                Tmdb::new(&self.http, &token).details(external_id).await
            }
            MediaType::Album | MediaType::Artist => {
                MusicBrainz::new(&self.http, &self.music_limiter)
                    .details(external_id)
                    .await
            }
            MediaType::Anime | MediaType::Manga => {
                AniList::new(&self.http).details(external_id).await
            }
        }
    }

    fn tmdb_token(&self) -> AppResult<String> {
        self.secrets.get(TMDB_TOKEN)?.ok_or(AppError::Invalid(
            "add a TMDB read access token in Settings to search movies and shows",
        ))
    }

    fn games_key(&self) -> AppResult<String> {
        self.secrets.get(GAMES_KEY)?.ok_or(AppError::Invalid(
            "add a RAWG API key in Settings to search games",
        ))
    }
}

/// A fresh cached answer wins; a failed fetch falls back to an old one, so searching works offline.
async fn cached<T: Serialize + DeserializeOwned>(
    db: &Db,
    key: &str,
    ttl_ms: i64,
    fetch: impl Future<Output = AppResult<T>>,
) -> AppResult<T> {
    let now = now_ms();
    let stored = metadata_cache::get(db, key).await?;
    if let Some(Cached {
        payload,
        fetched_at,
    }) = &stored
        && now - fetched_at < ttl_ms
        && let Ok(value) = serde_json::from_str(payload)
    {
        return Ok(value);
    }
    match fetch.await {
        Ok(value) => {
            metadata_cache::put(db, key, &serde_json::to_string(&value)?, now).await?;
            Ok(value)
        }
        Err(error) => stored
            .and_then(|old| serde_json::from_str(&old.payload).ok())
            .ok_or(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::pacing::tests::FakeClock;
    use crate::media::test_support::{FakeHttp, MemorySecrets};

    const ANILIST: &str = include_str!("fixtures/anilist_search.json");
    const RAWG: &str = include_str!("fixtures/rawg_search.json");
    const TMDB: &str = include_str!("fixtures/tmdb_search_movie.json");
    const MUSIC: &str = include_str!("fixtures/musicbrainz_release_groups.json");

    fn service(http: FakeHttp) -> MetadataService<FakeHttp, MemorySecrets, FakeClock> {
        MetadataService::new(http, MemorySecrets::default(), FakeClock::default())
    }

    #[tokio::test]
    async fn a_repeated_search_is_answered_from_the_cache() {
        let db = Db::in_memory().await;
        let service = service(FakeHttp::answering(200, ANILIST));

        let first = service
            .search(&db, " Frieren ", MediaType::Anime)
            .await
            .expect("first");
        let second = service
            .search(&db, "frieren", MediaType::Anime)
            .await
            .expect("second");

        assert_eq!(first, second);
        assert_eq!(service.http.requests.borrow().len(), 1);
    }

    #[tokio::test]
    async fn anime_and_manga_searches_are_cached_apart() {
        let db = Db::in_memory().await;
        let http = FakeHttp::answering(200, ANILIST);
        http.push(200, ANILIST);
        let service = service(http);

        service
            .search(&db, "x", MediaType::Anime)
            .await
            .expect("anime");
        service
            .search(&db, "x", MediaType::Manga)
            .await
            .expect("manga");

        assert_eq!(service.http.requests.borrow().len(), 2);
    }

    #[tokio::test]
    async fn an_old_cached_answer_is_used_when_the_network_fails() {
        let db = Db::in_memory().await;
        let service = service(FakeHttp::answering(200, ANILIST));
        let fresh = service
            .search(&db, "frieren", MediaType::Anime)
            .await
            .expect("fresh");
        sqlx::query("UPDATE metadata_cache SET fetched_at = 0")
            .execute(&db.pool)
            .await
            .expect("age the cache");
        service.http.push_failure();

        let offline = service
            .search(&db, "frieren", MediaType::Anime)
            .await
            .expect("stale answer");

        assert_eq!(offline, fresh);
        assert_eq!(service.http.requests.borrow().len(), 2);
    }

    #[tokio::test]
    async fn without_a_cache_a_failure_is_reported() {
        let db = Db::in_memory().await;
        let http = FakeHttp::default();
        http.push_failure();
        let service = service(http);

        let result = service.search(&db, "frieren", MediaType::Anime).await;

        assert!(matches!(result, Err(AppError::Provider(_))));
    }

    #[tokio::test]
    async fn failed_answers_are_not_cached() {
        let db = Db::in_memory().await;
        let http = FakeHttp::answering(429, "{}");
        http.push(200, ANILIST);
        let service = service(http);

        assert!(service.search(&db, "a", MediaType::Anime).await.is_err());
        let hits = service
            .search(&db, "a", MediaType::Anime)
            .await
            .expect("retry");

        assert_eq!(hits.len(), 2);
    }

    #[tokio::test]
    async fn blank_queries_make_no_request() {
        let db = Db::in_memory().await;
        let service = service(FakeHttp::default());

        let hits = service
            .search(&db, "   ", MediaType::Anime)
            .await
            .expect("blank");

        assert!(hits.is_empty());
        assert!(service.http.requests.borrow().is_empty());
    }

    #[tokio::test]
    async fn games_need_a_key_and_send_it_to_rawg() {
        let db = Db::in_memory().await;
        let service = service(FakeHttp::answering(200, RAWG));

        let missing = service.search(&db, "hollow", MediaType::Game).await;
        assert!(matches!(missing, Err(AppError::Invalid(_))));
        assert!(service.http.requests.borrow().is_empty());

        service.set_games_key(" abc123 ").expect("key");
        let hits = service
            .search(&db, "hollow", MediaType::Game)
            .await
            .expect("search");

        assert_eq!(hits.len(), 2);
        let requests = service.http.requests.borrow();
        assert!(requests[0].url.starts_with("https://api.rawg.io/"));
        assert!(requests[0].params.contains(&("key", "abc123".to_string())));
    }

    #[tokio::test]
    async fn music_searches_are_paced_and_cached() {
        let db = Db::in_memory().await;
        let http = FakeHttp::answering(200, MUSIC);
        http.push(200, MUSIC);
        let service = service(http);

        for query in ["radiohead", "radiohead", "ok computer"] {
            service
                .search(&db, query, MediaType::Album)
                .await
                .expect("search");
        }

        assert_eq!(service.http.requests.borrow().len(), 2);
        assert_eq!(service.music_limiter.clock().sleeps.borrow().len(), 1);
    }

    #[tokio::test]
    async fn movies_need_a_token_and_send_it_as_a_bearer_header() {
        let db = Db::in_memory().await;
        let service = service(FakeHttp::answering(200, TMDB));

        let missing = service.search(&db, "matrix", MediaType::Movie).await;
        assert!(matches!(missing, Err(AppError::Invalid(_))));
        assert!(service.http.requests.borrow().is_empty());

        service.set_tmdb_token(" tok123 ").expect("token");
        let hits = service
            .search(&db, "matrix", MediaType::Movie)
            .await
            .expect("search");

        assert_eq!(hits.len(), 2);
        let requests = service.http.requests.borrow();
        assert_eq!(
            requests[0].headers,
            [("authorization", "Bearer tok123".to_string())]
        );
    }

    #[test]
    fn the_tmdb_token_can_be_set_checked_and_cleared() {
        let service = service(FakeHttp::default());
        assert!(!service.has_tmdb_token().expect("status"));

        service.set_tmdb_token("tok").expect("set");
        assert!(service.has_tmdb_token().expect("status"));
        service.clear_tmdb_token().expect("clear");
        assert!(!service.has_tmdb_token().expect("status"));
        assert!(service.set_tmdb_token("a b").is_err());
    }

    #[test]
    fn the_games_key_can_be_set_checked_and_cleared() {
        let service = service(FakeHttp::default());
        assert!(!service.has_games_key().expect("status"));

        service.set_games_key("abc123").expect("set");
        assert!(service.has_games_key().expect("status"));
        service.clear_games_key().expect("clear");
        assert!(!service.has_games_key().expect("status"));

        assert!(service.set_games_key("  ").is_err());
        assert!(service.set_games_key("a b").is_err());
    }

    #[tokio::test]
    async fn details_are_cached_per_source_and_id() {
        let db = Db::in_memory().await;
        let body = r#"{"data":{"Media":{"id":7,"title":{"romaji":"Seven"}}}}"#;
        let service = service(FakeHttp::answering(200, body));

        let first = service
            .details(&db, MediaType::Manga, "7")
            .await
            .expect("first");
        let second = service
            .details(&db, MediaType::Anime, "7")
            .await
            .expect("second");

        assert_eq!(first, second);
        assert_eq!(first.details.title, "Seven");
        assert_eq!(service.http.requests.borrow().len(), 1);
    }
}
