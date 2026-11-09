use crate::db;
use crate::db::settings::ensure_device_id;
use crate::error::AppResult;
use crate::sync::hlc::Hlc;
use std::path::PathBuf;

const DB_FILE: &str = "kenchamios.db";

pub struct AppState {
    pub device_id: String,
    pub db_path: PathBuf,
}

impl AppState {
    pub async fn init(data_dir: PathBuf) -> AppResult<Self> {
        let db_path = data_dir.join(DB_FILE);
        let pool = db::open(&db_path).await?;
        let device_id = ensure_device_id(&pool, &Hlc::new()).await?;
        Ok(Self { device_id, db_path })
    }
}
