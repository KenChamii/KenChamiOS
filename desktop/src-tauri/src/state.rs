use crate::db::settings::ensure_device_id;
use crate::db::{self, Db, templates};
use crate::error::AppResult;
use crate::sync::hlc::Hlc;
use std::path::PathBuf;

const DB_FILE: &str = "kenchamios.db";

pub struct AppState {
    pub db: Db,
    pub db_path: PathBuf,
}

impl AppState {
    pub async fn init(data_dir: PathBuf) -> AppResult<Self> {
        let db_path = data_dir.join(DB_FILE);
        let pool = db::open(&db_path).await?;
        let hlc = Hlc::new();
        let device_id = ensure_device_id(&pool, &hlc).await?;
        let db = Db::new(pool, hlc, device_id);
        templates::ensure_defaults(&db).await?;
        Ok(Self { db, db_path })
    }
}
