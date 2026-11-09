use crate::db::settings::ensure_device_id;
use crate::db::{self, Db, templates};
use crate::error::AppResult;
use crate::media::http::ReqwestClient;
use crate::media::secrets::KeyringStore;
use crate::media::service::MetadataService;
use crate::sync::hlc::Hlc;
use crate::timer::service::TimerService;
use std::path::PathBuf;

const DB_FILE: &str = "kenchamios.db";
const ASSETS_DIR: &str = "assets";
const COVERS_DIR: &str = "covers";

pub struct AppState {
    pub db: Db,
    pub db_path: PathBuf,
    pub assets_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub timer: TimerService,
    pub metadata: MetadataService<ReqwestClient, KeyringStore>,
}

impl AppState {
    pub async fn init(data_dir: PathBuf) -> AppResult<Self> {
        let db_path = data_dir.join(DB_FILE);
        let pool = db::open(&db_path).await?;
        let hlc = Hlc::new();
        let device_id = ensure_device_id(&pool, &hlc).await?;
        let db = Db::new(pool, hlc, device_id);
        templates::ensure_defaults(&db).await?;
        let timer = TimerService::recover(&db).await?;
        let assets_dir = data_dir.join(ASSETS_DIR);
        let covers_dir = data_dir.join(COVERS_DIR);
        let metadata = MetadataService::new(ReqwestClient::new()?, KeyringStore);
        Ok(Self {
            db,
            db_path,
            assets_dir,
            covers_dir,
            timer,
            metadata,
        })
    }
}
