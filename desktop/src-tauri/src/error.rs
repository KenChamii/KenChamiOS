use serde::{Serialize, Serializer, ser::SerializeStruct};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("invalid stored value: {0}")]
    Json(#[from] serde_json::Error),
    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("{0}")]
    Invalid(&'static str),
    #[error("{0}")]
    Provider(String),
    #[error("credential store error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("app error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("sync error: {0}")]
    Sync(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "database",
            Self::Migration(_) => "migration",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::Git(_) => "git",
            Self::Template(_) => "template",
            Self::Tauri(_) => "tauri",
            Self::Provider(_) => "provider",
            Self::Keyring(_) => "keyring",
            Self::Sync(_) => "sync",
            Self::NotFound(_) => "not_found",
            Self::Invalid(_) => "invalid",
        }
    }
}

// the ui gets { code, message }
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
