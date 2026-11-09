use super::lan;
use super::limiter::FailureLimiter;
use super::mdns::Advertisement;
use super::pairing::PairingWindow;
use super::routes::ServerState;
use super::schema::SyncSchema;
use super::server::{self, RunningServer};
use super::status::{Notifier, StatusError, SyncStatus};
use super::tls::Identity;
use super::types::PairingPayload;
use crate::db::Db;
use crate::db::paired_devices::{self, PairedDevice};
use crate::db::settings;
use crate::error::{AppError, AppResult};
use crate::media::secrets::SecretStore;
use serde::Serialize;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

const ENABLED_KEY: &str = "sync.enabled";
const DEFAULT_PORT: u16 = 47821;
const BACKUPS_DIR: &str = "backups";

struct Running {
    server: RunningServer,
    advertisement: Option<Advertisement>,
    fingerprint: String,
}

#[derive(Debug, Serialize)]
pub struct SyncView {
    pub enabled: bool,
    pub running: bool,
    pub address: Option<String>,
    pub fingerprint: Option<String>,
    pub pairing_open: bool,
    pub last_error: Option<StatusError>,
    pub devices: Vec<PairedDevice>,
}

#[derive(Debug, Serialize)]
pub struct PairingInfo {
    pub qr_svg: String,
    pub payload: PairingPayload,
    pub expires_in_s: u64,
}

/// Owns the sync server: off until the user turns it on, one pairing window at a time.
pub struct SyncService {
    state: ServerState,
    secrets: Arc<dyn SecretStore + Send + Sync>,
    running: Mutex<Option<Running>>,
}

impl SyncService {
    pub fn new(
        db: Arc<Db>,
        secrets: Arc<dyn SecretStore + Send + Sync>,
        data_dir: PathBuf,
    ) -> AppResult<Self> {
        let state = ServerState {
            db,
            schema: Arc::new(SyncSchema::parse()?),
            pairing: Arc::new(PairingWindow::default()),
            limiter: Arc::new(FailureLimiter::default()),
            status: Arc::new(SyncStatus::default()),
            notifier: Arc::new(Notifier::default()),
            backups_dir: Arc::new(data_dir.join(BACKUPS_DIR)),
        };
        Ok(Self {
            state,
            secrets,
            running: Mutex::new(None),
        })
    }

    pub fn on_data_changed(&self, callback: impl Fn() + Send + Sync + 'static) {
        self.state.notifier.set(callback);
    }

    /// Starts the server at app launch when sync was left on; a failure shows up as the status error.
    pub async fn resume(&self) {
        if self.is_enabled().await.unwrap_or(false)
            && let Err(error) = self.start_on_lan().await
        {
            self.state.status.record_error(&error.to_string());
        }
    }

    async fn is_enabled(&self) -> AppResult<bool> {
        let stored = settings::get_string(&self.state.db, ENABLED_KEY).await?;
        Ok(stored.as_deref() == Some("true"))
    }

    pub async fn set_enabled(&self, enabled: bool) -> AppResult<()> {
        if enabled {
            self.start_on_lan().await?;
        } else {
            self.stop().await;
        }
        settings::set_string(
            &self.state.db,
            ENABLED_KEY,
            if enabled { "true" } else { "false" },
        )
        .await
    }

    async fn start_on_lan(&self) -> AppResult<()> {
        let ip = lan::private_ipv4().ok_or_else(|| {
            AppError::Sync("no home network address found, connect to Wi-Fi first".to_string())
        })?;
        self.start_on(SocketAddr::new(IpAddr::V4(ip), DEFAULT_PORT), true)
            .await
            .map(|_| ())
    }

    /// Starts the server on `bind` unless it is already running; returns the bound address.
    pub async fn start_on(&self, bind: SocketAddr, advertise: bool) -> AppResult<SocketAddr> {
        let mut running = self.running.lock().await;
        if let Some(current) = running.as_ref() {
            return Ok(current.server.addr);
        }
        let identity = Identity::load_or_create(self.secrets.as_ref())?;
        let server = server::start(self.state.clone(), identity.server_config()?, bind)?;
        let advertisement = match (advertise, server.addr.ip()) {
            (true, IpAddr::V4(ip)) => Some(Advertisement::start(ip, server.addr.port())?),
            _ => None,
        };
        let addr = server.addr;
        *running = Some(Running {
            server,
            advertisement,
            fingerprint: identity.fingerprint(),
        });
        self.state.status.clear();
        Ok(addr)
    }

    pub async fn stop(&self) {
        self.state.pairing.close();
        if let Some(current) = self.running.lock().await.take() {
            current.server.stop();
            if let Some(advertisement) = current.advertisement {
                advertisement.stop();
            }
        }
    }

    /// Opens the pairing window and returns what the QR code shows.
    pub async fn start_pairing(&self) -> AppResult<PairingInfo> {
        let running = self.running.lock().await;
        let current = running
            .as_ref()
            .ok_or_else(|| AppError::Sync("turn sync on before pairing a device".to_string()))?;
        let now = Instant::now();
        let (code, expires) = self.state.pairing.open(now)?;
        let payload = PairingPayload {
            v: 1,
            host: current.server.addr.ip().to_string(),
            port: current.server.addr.port(),
            fp: current.fingerprint.clone(),
            code,
        };
        Ok(PairingInfo {
            qr_svg: qr_svg(&payload)?,
            payload,
            expires_in_s: expires.duration_since(now).as_secs(),
        })
    }

    pub fn cancel_pairing(&self) {
        self.state.pairing.close();
    }

    pub async fn revoke(&self, device_id: &str) -> AppResult<()> {
        paired_devices::revoke(&self.state.db, device_id).await
    }

    pub async fn view(&self) -> AppResult<SyncView> {
        let running = self.running.lock().await;
        Ok(SyncView {
            enabled: self.is_enabled().await?,
            running: running.is_some(),
            address: running.as_ref().map(|r| r.server.addr.to_string()),
            fingerprint: running.as_ref().map(|r| r.fingerprint.clone()),
            pairing_open: self.state.pairing.is_open(Instant::now()),
            last_error: self.state.status.last_error(),
            devices: paired_devices::list(&self.state.db).await?,
        })
    }
}

fn qr_svg(payload: &PairingPayload) -> AppResult<String> {
    let text = serde_json::to_string(payload)?;
    let code = qrcode::QrCode::new(text.as_bytes())
        .map_err(|e| AppError::Sync(format!("could not draw the QR code: {e}")))?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(220, 220)
        .build())
}
