use crate::db::settings::ensure_device_id;
use crate::db::{self, Db, templates};
use crate::error::AppResult;
use crate::media::http::ReqwestClient;
use crate::media::pacing::SystemClock;
use crate::media::secrets::KeyringStore;
use crate::media::service::MetadataService;
use crate::sync::SyncService;
use crate::sync::hlc::{self, Hlc};
use crate::timer::service::TimerService;
use std::path::PathBuf;
use std::sync::Arc;

const DB_FILE: &str = "kenchamios.db";
const ASSETS_DIR: &str = "assets";
const COVERS_DIR: &str = "covers";

pub struct AppState {
    pub db: Arc<Db>,
    pub db_path: PathBuf,
    pub assets_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub timer: TimerService,
    pub metadata: MetadataService<ReqwestClient, KeyringStore, SystemClock>,
    pub sync: SyncService,
}

impl AppState {
    pub async fn init(data_dir: PathBuf) -> AppResult<Self> {
        let db_path = data_dir.join(DB_FILE);
        let pool = db::open(&db_path).await?;
        let hlc = Hlc::new();
        hlc::restore(&pool, &hlc).await?;
        let device_id = ensure_device_id(&pool, &hlc).await?;
        let db = Arc::new(Db::new(pool, hlc, device_id));
        templates::ensure_defaults(&db).await?;
        let timer = TimerService::recover(&db).await?;
        let assets_dir = data_dir.join(ASSETS_DIR);
        let covers_dir = data_dir.join(COVERS_DIR);
        let metadata =
            MetadataService::new(ReqwestClient::new()?, KeyringStore, SystemClock::default());
        let sync = SyncService::new(db.clone(), Arc::new(KeyringStore), data_dir)?;
        Ok(Self {
            db,
            db_path,
            assets_dir,
            covers_dir,
            timer,
            metadata,
            sync,
        })
    }
}
