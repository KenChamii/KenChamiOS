use super::protocol;
use crate::db::{Db, now_ms};
use crate::error::{AppError, AppResult};
use crate::sync::codes::secrets_equal;
use axum::Router;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde_json::{Value, json};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

const MAX_BODY: usize = 64 * 1024;

#[derive(Clone)]
struct McpState {
    db: Arc<Db>,
    token: Arc<String>,
}

/// The localhost listener; dropping it stops the server.
pub struct Running {
    pub addr: SocketAddr,
    stop: Option<oneshot::Sender<()>>,
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

fn router(state: McpState) -> Router {
    Router::new()
        .route("/mcp", post(rpc))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .with_state(state)
}

/// A browser page on another site could try to reach 127.0.0.1; only local origins pass.
fn origin_allowed(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else {
        return true;
    };
    let origin = origin.to_str().unwrap_or("");
    [
        "http://127.0.0.1",
        "http://localhost",
        "https://localhost",
        "tauri://localhost",
    ]
    .iter()
    .any(|ok| origin == *ok || origin.starts_with(&format!("{ok}:")))
}

fn authorized(headers: &HeaderMap, token: &str) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|given| secrets_equal(given, token))
}

async fn rpc(State(state): State<McpState>, headers: HeaderMap, body: String) -> Response {
    if !origin_allowed(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Ok(request) = serde_json::from_str::<Value>(&body) else {
        let error = json!({ "jsonrpc": "2.0", "id": null,
                            "error": { "code": -32700, "message": "invalid JSON" } });
        return json_response(StatusCode::BAD_REQUEST, &error);
    };
    match protocol::handle(&state.db, &request, now_ms()).await {
        Some(answer) => json_response(StatusCode::OK, &answer),
        None => StatusCode::ACCEPTED.into_response(),
    }
}

fn json_response(status: StatusCode, body: &Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response()
}

/// Starts listening on 127.0.0.1 only. Port 0 picks a free one (tests).
pub async fn start(db: Arc<Db>, token: String, port: u16) -> AppResult<Running> {
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .await
        .map_err(|e| {
            AppError::Agent(format!(
                "could not start the MCP server on port {port}: {e}"
            ))
        })?;
    let addr = listener.local_addr()?;
    let app = router(McpState {
        db,
        token: Arc::new(token),
    });
    let (stop, stopped) = oneshot::channel::<()>();
    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await;
    });
    Ok(Running {
        addr,
        stop: Some(stop),
    })
}

/// Holds the running server, so the settings can start, stop and restart it.
#[derive(Default)]
pub struct McpService {
    running: Mutex<Option<Running>>,
}

impl McpService {
    pub fn is_running(&self) -> bool {
        self.port().is_some()
    }

    /// The port it listens on, while it runs.
    pub fn port(&self) -> Option<u16> {
        let slot = self.running.lock().ok()?;
        slot.as_ref().map(|running| running.addr.port())
    }

    pub async fn start(&self, db: Arc<Db>, token: String, port: u16) -> AppResult<()> {
        self.stop();
        let running = start(db, token, port).await?;
        if let Ok(mut slot) = self.running.lock() {
            *slot = Some(running);
        }
        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut slot) = self.running.lock() {
            *slot = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;

    async fn post_json(
        addr: SocketAddr,
        token: Option<&str>,
        origin: Option<&str>,
        body: &str,
    ) -> (u16, String) {
        let client = reqwest::Client::new();
        let mut request = client
            .post(format!("http://{addr}/mcp"))
            .body(body.to_string());
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        let response = request.send().await.expect("send");
        let status = response.status().as_u16();
        (status, response.text().await.expect("text"))
    }

    #[tokio::test]
    async fn the_server_needs_the_token_and_a_local_origin() {
        let db = Arc::new(Db::in_memory().await);
        let running = start(db, "secret-token".into(), 0).await.expect("start");
        let list = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;

        assert_eq!(post_json(running.addr, None, None, list).await.0, 401);
        assert_eq!(
            post_json(running.addr, Some("wrong"), None, list).await.0,
            401
        );
        assert_eq!(
            post_json(
                running.addr,
                Some("secret-token"),
                Some("https://evil.example"),
                list
            )
            .await
            .0,
            403
        );
        let (status, body) = post_json(running.addr, Some("secret-token"), None, list).await;
        assert_eq!(status, 200);
        assert!(body.contains("read_today"));
        let local = post_json(
            running.addr,
            Some("secret-token"),
            Some("http://localhost:3000"),
            list,
        )
        .await;
        assert_eq!(local.0, 200);
    }

    #[tokio::test]
    async fn notifications_are_accepted_and_bad_json_is_refused() {
        let db = Arc::new(Db::in_memory().await);
        let running = start(db, "t".into(), 0).await.expect("start");

        let note = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        assert_eq!(post_json(running.addr, Some("t"), None, note).await.0, 202);
        assert_eq!(
            post_json(running.addr, Some("t"), None, "{nope").await.0,
            400
        );
    }

    #[tokio::test]
    async fn a_call_over_http_changes_the_app() {
        let db = Arc::new(Db::in_memory().await);
        let project = create_for_test(&db, "Demo").await;
        let running = start(db.clone(), "t".into(), 0).await.expect("start");

        let call = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"add_task","arguments":{"project":"Demo","title":"From Claude"}}}"#;
        let (status, body) = post_json(running.addr, Some("t"), None, call).await;

        assert_eq!(status, 200, "{body}");
        let tasks = crate::db::tasks::list(&db, &project.id)
            .await
            .expect("tasks");
        assert_eq!(tasks[0].title, "From Claude");
    }

    #[tokio::test]
    async fn dropping_the_handle_stops_the_server() {
        let db = Arc::new(Db::in_memory().await);
        let running = start(db, "t".into(), 0).await.expect("start");
        let addr = running.addr;
        drop(running);
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let client = reqwest::Client::new();
        let result = client
            .post(format!("http://{addr}/mcp"))
            .body("{}")
            .send()
            .await;
        assert!(result.is_err(), "the port is closed");
    }
}

#[cfg(test)]
mod manual {
    use super::*;
    use crate::db::projects::create_for_test;

    // run by hand: `cargo test --lib mcp::server::manual -- --ignored`, then point a real
    // MCP client at port 47999 with the token "manual-check-token"
    #[tokio::test]
    #[ignore = "serves for two minutes so a real client can connect"]
    async fn serve_for_a_real_client() {
        let db = Arc::new(Db::in_memory().await);
        create_for_test(&db, "Demo App").await;
        let _running = start(db.clone(), "manual-check-token".into(), 47_999)
            .await
            .expect("start");
        tokio::time::sleep(std::time::Duration::from_secs(120)).await;
        let calls = crate::db::mcp_calls::list(&db, 10).await.expect("calls");
        for call in calls {
            println!(
                "{} ok={} {}",
                call.call.tool, call.call.ok, call.call.summary
            );
        }
    }
}
