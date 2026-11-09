use super::routes::{ServerState, router};
use crate::error::{AppError, AppResult};
use axum_server::Handle;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

const HEADER_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_GRACE: Duration = Duration::from_secs(2);

/// The HTTPS listener; dropping the value does not stop it, `stop` does.
pub struct RunningServer {
    handle: Handle<SocketAddr>,
    pub addr: SocketAddr,
}

fn bind_error(error: std::io::Error) -> AppError {
    AppError::Sync(format!("could not start the sync server: {error}"))
}

pub fn start(
    state: ServerState,
    tls: Arc<rustls::ServerConfig>,
    bind: SocketAddr,
) -> AppResult<RunningServer> {
    let listener = std::net::TcpListener::bind(bind).map_err(bind_error)?;
    let addr = listener.local_addr().map_err(bind_error)?;
    let mut server =
        axum_server::tls_rustls::from_tcp_rustls(listener, RustlsConfig::from_config(tls))
            .map_err(bind_error)?
            .http1_only();
    server
        .http_builder()
        .http1()
        .header_read_timeout(HEADER_TIMEOUT);
    let handle = Handle::new();
    let status = state.status.clone();
    let service = router(state).into_make_service_with_connect_info::<SocketAddr>();
    let server = server.handle(handle.clone());
    tokio::spawn(async move {
        if server.serve(service).await.is_err() {
            status.record_error("the sync server stopped unexpectedly");
        }
    });
    Ok(RunningServer { handle, addr })
}

impl RunningServer {
    pub fn stop(&self) {
        self.handle.graceful_shutdown(Some(STOP_GRACE));
    }
}
