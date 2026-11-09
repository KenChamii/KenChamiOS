use crate::error::{AppError, AppResult};
use std::time::Duration;

// MusicBrainz asks for an app name, version and a contact url; no personal data
const USER_AGENT: &str = concat!(
    "KenChamiOS/",
    env!("CARGO_PKG_VERSION"),
    " ( https://github.com/KenChamii/KenChamiOS )"
);
const TIMEOUT: Duration = Duration::from_secs(15);

/// A GET, or a JSON POST when `body` is set. Params and headers may hold a key, so no `Debug`.
#[derive(Clone)]
pub struct HttpRequest {
    pub url: String,
    pub params: Vec<(&'static str, String)>,
    pub headers: Vec<(&'static str, String)>,
    pub body: Option<String>,
    pub max_bytes: usize,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub retry_after_s: Option<u64>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// The body of a successful answer; other statuses become a readable error.
    pub fn into_body(self) -> AppResult<Vec<u8>> {
        match self.status {
            200..=299 => Ok(self.body),
            429 => Err(AppError::Provider(match self.retry_after_s {
                Some(seconds) => format!("rate limited, try again in {seconds} seconds"),
                None => "rate limited, try again in a minute".to_string(),
            })),
            401 | 403 => Err(AppError::Provider(
                "the service refused the request, check the API key".to_string(),
            )),
            status => Err(AppError::Provider(format!(
                "the service answered with status {status}"
            ))),
        }
    }
}

/// The only place that talks to the network, so tests can swap it for recorded answers.
pub trait HttpClient {
    async fn send(&self, request: HttpRequest) -> AppResult<HttpResponse>;
}

pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .build()
            .map_err(network_error)?;
        Ok(Self { client })
    }
}

impl HttpClient for ReqwestClient {
    async fn send(&self, request: HttpRequest) -> AppResult<HttpResponse> {
        let url = reqwest::Url::parse_with_params(&request.url, &request.params)
            .map_err(|_| AppError::Invalid("the request address is not valid"))?;
        let mut builder = match request.body {
            Some(body) => self
                .client
                .post(url)
                .header("content-type", "application/json")
                .body(body),
            None => self.client.get(url),
        };
        for (name, value) in &request.headers {
            builder = builder.header(*name, value);
        }
        let mut response = builder.send().await.map_err(network_error)?;
        let status = response.status().as_u16();
        let retry_after_s = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse().ok());
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(network_error)? {
            if body.len() + chunk.len() > request.max_bytes {
                return Err(AppError::Provider("the answer is too large".to_string()));
            }
            body.extend_from_slice(&chunk);
        }
        Ok(HttpResponse {
            status,
            retry_after_s,
            body,
        })
    }
}

// the address can carry an api key, so it never goes into the message
fn network_error(error: reqwest::Error) -> AppError {
    let error = error.without_url();
    AppError::Provider(format!("could not reach the service: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u16, retry_after_s: Option<u64>) -> HttpResponse {
        HttpResponse {
            status,
            retry_after_s,
            body: b"{}".to_vec(),
        }
    }

    #[test]
    fn the_user_agent_names_the_app_and_has_no_personal_data() {
        assert!(USER_AGENT.starts_with("KenChamiOS/"));
        assert!(USER_AGENT.contains("https://github.com/KenChamii/KenChamiOS"));
        assert!(!USER_AGENT.contains('@'));
    }

    #[test]
    fn success_gives_the_body() {
        assert_eq!(response(200, None).into_body().expect("ok"), b"{}");
    }

    #[test]
    fn rate_limits_say_how_long_to_wait() {
        let error = response(429, Some(42)).into_body().expect_err("limited");
        assert!(error.to_string().contains("42 seconds"), "{error}");
        let error = response(429, None).into_body().expect_err("limited");
        assert!(error.to_string().contains("rate limited"), "{error}");
    }

    #[test]
    fn refusals_and_server_errors_are_readable() {
        let denied = response(401, None).into_body().expect_err("denied");
        assert!(denied.to_string().contains("API key"));
        let broken = response(503, None).into_body().expect_err("broken");
        assert!(broken.to_string().contains("503"));
    }
}
