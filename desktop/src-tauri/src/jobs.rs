use crate::db::due_reminders::{self, DueReminder};
use crate::db::{backups, now_ms};
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

const APP_NAME: &str = "KenChamiOs";
const EVERY: Duration = Duration::from_secs(60);
// more due tasks than this become one summary notification
const MAX_SEPARATE: usize = 3;

/// Background work once a minute: due task reminders and the daily backup.
pub fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            run_once(&app).await;
            tokio::time::sleep(EVERY).await;
        }
    });
}

async fn run_once(app: &AppHandle) {
    let state = app.state::<AppState>();
    match due_reminders::take(&state.db, now_ms()).await {
        Ok(due) => notify_due(app, &due),
        Err(error) => eprintln!("could not read due tasks: {error}"),
    }
    if let Err(error) = backups::run_if_due(&state.db, &state.default_backups_dir).await {
        eprintln!("daily backup failed: {error}");
    }
}

fn notify_due(app: &AppHandle, due: &[DueReminder]) {
    let bodies: Vec<String> = if due.len() > MAX_SEPARATE {
        vec![format!("{} tasks are due today", due.len())]
    } else {
        due.iter()
            .map(|task| format!("Due today: {} ({})", task.title, task.project_name))
            .collect()
    };
    for body in bodies {
        let result = app
            .notification()
            .builder()
            .title(APP_NAME)
            .body(body)
            .show();
        if let Err(error) = result {
            eprintln!("could not show the notification: {error}");
        }
    }
}
