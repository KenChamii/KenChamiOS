use crate::domain::media::{ItemDetails, MediaType};
use crate::error::AppResult;
use serde::{Deserialize, Serialize};

/// One title as a metadata source describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub external_id: String,
    #[serde(flatten)]
    pub details: ItemDetails,
}

/// A metadata source; each one is used through its own concrete type, so no trait objects.
pub trait MetadataProvider {
    async fn search(&self, query: &str, media_type: MediaType) -> AppResult<Vec<Metadata>>;
    async fn details(&self, external_id: &str) -> AppResult<Metadata>;
}

/// The next episode of a title that is still airing; the time is when it is released.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Airing {
    pub external_id: String,
    pub episode: i64,
    pub air_at_ms: i64,
}
