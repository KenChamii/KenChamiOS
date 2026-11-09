use crate::db::now_ms;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize)]
pub struct StatusError {
    pub at: i64,
    pub message: String,
}

/// What the sync screen shows besides the paired devices: the latest problem.
#[derive(Default)]
pub struct SyncStatus {
    last_error: Mutex<Option<StatusError>>,
}

impl SyncStatus {
    pub fn record_error(&self, message: &str) {
        if let Ok(mut last) = self.last_error.lock() {
            *last = Some(StatusError {
                at: now_ms(),
                message: message.to_string(),
            });
        }
    }

    pub fn clear(&self) {
        if let Ok(mut last) = self.last_error.lock() {
            *last = None;
        }
    }

    pub fn last_error(&self) -> Option<StatusError> {
        self.last_error.lock().ok().and_then(|last| last.clone())
    }
}

type Callback = Box<dyn Fn() + Send + Sync>;

/// Tells the window that synced data arrived; set once at startup.
#[derive(Default)]
pub struct Notifier(OnceLock<Callback>);

impl Notifier {
    pub fn set(&self, callback: impl Fn() + Send + Sync + 'static) {
        let _ = self.0.set(Box::new(callback));
    }

    pub fn notify(&self) {
        if let Some(callback) = self.0.get() {
            callback();
        }
    }
}
